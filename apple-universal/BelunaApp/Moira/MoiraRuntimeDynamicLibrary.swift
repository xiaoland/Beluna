#if os(macOS)
import Darwin
import Foundation

final class MoiraRuntimeDynamicLibrary: @unchecked Sendable {
    private typealias StatusFunction = @convention(c) (
        UnsafePointer<CChar>,
        UnsafePointer<CChar>,
        UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
        UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
    ) -> Int32
    private typealias LoomFunction = @convention(c) (
        UnsafePointer<CChar>,
        UnsafePointer<CChar>,
        UnsafePointer<CChar>,
        UnsafePointer<CChar>,
        UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
        UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
    ) -> Int32
    private typealias ShutdownFunction = @convention(c) (
        UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
        UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
    ) -> Int32
    private typealias FreeFunction = @convention(c) (UnsafeMutablePointer<CChar>?) -> Void

    private let handle: UnsafeMutableRawPointer
    private let statusFunction: StatusFunction
    private let loomFunction: LoomFunction
    private let shutdownFunction: ShutdownFunction
    private let freeFunction: FreeFunction

    private init(
        handle: UnsafeMutableRawPointer,
        statusFunction: @escaping StatusFunction,
        loomFunction: @escaping LoomFunction,
        shutdownFunction: @escaping ShutdownFunction,
        freeFunction: @escaping FreeFunction
    ) {
        self.handle = handle
        self.statusFunction = statusFunction
        self.loomFunction = loomFunction
        self.shutdownFunction = shutdownFunction
        self.freeFunction = freeFunction
    }

    deinit {
        dlclose(handle)
    }

    static func loadDefault(filePath: StaticString = #filePath) throws -> Self {
        try load(candidateURLs: candidateLibraryURLs(filePath: filePath))
    }

    static func loadBundled() throws -> Self {
        guard let frameworksURL = Bundle.main.privateFrameworksURL else {
            throw MoiraRuntimeDynamicClientError.libraryMissing(["Bundle.main.privateFrameworksURL"])
        }

        return try load(candidateURLs: [
            frameworksURL.appendingPathComponent("libmoira_ffi.dylib"),
        ])
    }

    func statusSnapshot(configuration: MoiraRuntimeConfiguration) throws -> MoiraRuntimeSnapshot {
        var jsonPointer: UnsafeMutablePointer<CChar>?
        var errorPointer: UnsafeMutablePointer<CChar>?
        defer {
            freeFunction(jsonPointer)
            freeFunction(errorPointer)
        }

        let statusCode = configuration.rootDirectoryPath.withCString { rootPointer in
            configuration.receiverBind.withCString { bindPointer in
                statusFunction(rootPointer, bindPointer, &jsonPointer, &errorPointer)
            }
        }

        if statusCode != 0 {
            throw MoiraRuntimeDynamicClientError.statusFailure(readCString(errorPointer))
        }

        guard let jsonPointer else {
            throw MoiraRuntimeDynamicClientError.missingStatusPayload
        }

        let jsonText = String(cString: jsonPointer)
        let data = Data(jsonText.utf8)

        do {
            return try JSONDecoder().decode(MoiraRuntimeSnapshot.self, from: data)
        } catch {
            throw MoiraRuntimeDynamicClientError.invalidStatusPayload(String(describing: error))
        }
    }

    func loomSnapshot(
        configuration: MoiraRuntimeConfiguration,
        selection: MoiraLoomSelection
    ) throws -> MoiraLoomSnapshot {
        var jsonPointer: UnsafeMutablePointer<CChar>?
        var errorPointer: UnsafeMutablePointer<CChar>?
        defer {
            freeFunction(jsonPointer)
            freeFunction(errorPointer)
        }

        let runID = selection.runID ?? ""
        let tick = selection.tick.map(String.init) ?? ""
        let statusCode = configuration.rootDirectoryPath.withCString { rootPointer in
            configuration.receiverBind.withCString { bindPointer in
                runID.withCString { runPointer in
                    tick.withCString { tickPointer in
                        loomFunction(
                            rootPointer,
                            bindPointer,
                            runPointer,
                            tickPointer,
                            &jsonPointer,
                            &errorPointer
                        )
                    }
                }
            }
        }

        if statusCode != 0 {
            throw MoiraRuntimeDynamicClientError.statusFailure(readCString(errorPointer))
        }

        guard let jsonPointer else {
            throw MoiraRuntimeDynamicClientError.missingStatusPayload
        }

        let jsonText = String(cString: jsonPointer)
        do {
            return try JSONDecoder().decode(MoiraLoomSnapshot.self, from: Data(jsonText.utf8))
        } catch {
            throw MoiraRuntimeDynamicClientError.invalidStatusPayload(String(describing: error))
        }
    }

    func shutdownResources() throws -> [MoiraResourceStatus] {
        var jsonPointer: UnsafeMutablePointer<CChar>?
        var errorPointer: UnsafeMutablePointer<CChar>?
        defer {
            freeFunction(jsonPointer)
            freeFunction(errorPointer)
        }

        let statusCode = shutdownFunction(&jsonPointer, &errorPointer)
        if statusCode != 0 {
            throw MoiraRuntimeDynamicClientError.statusFailure(readCString(errorPointer))
        }

        guard let jsonPointer else {
            throw MoiraRuntimeDynamicClientError.missingStatusPayload
        }

        let jsonText = String(cString: jsonPointer)
        do {
            return try JSONDecoder().decode([MoiraResourceStatus].self, from: Data(jsonText.utf8))
        } catch {
            throw MoiraRuntimeDynamicClientError.invalidStatusPayload(String(describing: error))
        }
    }

    private static func load(candidateURLs: [URL]) throws -> Self {
        var loadErrors: [String] = []

        for url in candidateURLs where FileManager.default.fileExists(atPath: url.path) {
            do {
                let library = try load(url: url)
                return library
            } catch let error as MoiraRuntimeDynamicClientError {
                switch error {
                case let .libraryLoadFailed(errors):
                    loadErrors.append(contentsOf: errors)
                case .libraryMissing, .symbolMissing, .statusFailure, .missingStatusPayload, .invalidStatusPayload:
                    throw error
                }
            } catch {
                throw error
            }
        }

        if !loadErrors.isEmpty {
            throw MoiraRuntimeDynamicClientError.libraryLoadFailed(loadErrors)
        }

        throw MoiraRuntimeDynamicClientError.libraryMissing(candidateURLs.map(\.path))
    }

    private static func load(url: URL) throws -> Self {
        guard let handle = dlopen(url.path, RTLD_NOW | RTLD_LOCAL) else {
            throw MoiraRuntimeDynamicClientError.libraryLoadFailed([
                "\(url.path): \(lastDynamicLoaderError())",
            ])
        }

        do {
            let statusFunction: StatusFunction = try loadSymbol(
                handle: handle,
                name: "moira_runtime_status_json"
            )
            let loomFunction: LoomFunction = try loadSymbol(
                handle: handle,
                name: "moira_runtime_loom_json"
            )
            let shutdownFunction: ShutdownFunction = try loadSymbol(
                handle: handle,
                name: "moira_runtime_shutdown_json"
            )
            let freeFunction: FreeFunction = try loadSymbol(
                handle: handle,
                name: "moira_runtime_string_free"
            )

            return Self(
                handle: handle,
                statusFunction: statusFunction,
                loomFunction: loomFunction,
                shutdownFunction: shutdownFunction,
                freeFunction: freeFunction
            )
        } catch {
            dlclose(handle)
            throw error
        }
    }

    private static func loadSymbol<T>(
        handle: UnsafeMutableRawPointer,
        name: String
    ) throws -> T {
        guard let symbol = dlsym(handle, name) else {
            throw MoiraRuntimeDynamicClientError.symbolMissing(name)
        }

        return unsafeBitCast(symbol, to: T.self)
    }

    private static func lastDynamicLoaderError() -> String {
        guard let error = dlerror() else {
            return "unknown loader error"
        }

        return String(cString: error)
    }

    private static func candidateLibraryURLs(filePath: StaticString) -> [URL] {
        var urls: [URL] = []

        if let overridePath = ProcessInfo.processInfo.environment["BELUNA_MOIRA_FFI_DYLIB"],
           !overridePath.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            urls.append(URL(fileURLWithPath: overridePath))
        }

        if let frameworksURL = Bundle.main.privateFrameworksURL {
            urls.append(frameworksURL.appendingPathComponent("libmoira_ffi.dylib"))
        }

        let sourceURL = URL(fileURLWithPath: "\(filePath)")
        let repoRoot = sourceURL
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
        urls.append(repoRoot.appendingPathComponent("target/debug/libmoira_ffi.dylib"))
        urls.append(repoRoot.appendingPathComponent("target/release/libmoira_ffi.dylib"))

        return urls
    }

    private func readCString(_ pointer: UnsafeMutablePointer<CChar>?) -> String {
        guard let pointer else {
            return "Moira FFI returned an empty error payload"
        }

        return String(cString: pointer)
    }
}
#endif
