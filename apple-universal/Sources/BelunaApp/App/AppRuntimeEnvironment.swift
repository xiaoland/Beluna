import Foundation

enum AppRuntimeEnvironment {
    static var isXcodeSession: Bool {
        let env = ProcessInfo.processInfo.environment
        if env["XCODE_RUNNING_FOR_PREVIEWS"] == "1" {
            return true
        }
        if env["__XCODE_BUILT_PRODUCTS_DIR_PATHS"] != nil {
            return true
        }
        if let serviceName = env["XPC_SERVICE_NAME"], serviceName.contains("Xcode") {
            return true
        }
        return false
    }

    static var needsManualActivationPolicy: Bool {
        !Bundle.main.bundlePath.hasSuffix(".app")
    }
}
