import AppKit
import Foundation
import Darwin

@MainActor
final class AppRuntimeGuard {
    static let shared = AppRuntimeGuard()

    private var lockFD: Int32?

    private init() {}

    func bootstrapOrTerminate() {
        guard acquireSingleInstanceLock() else {
            fputs("[BelunaApp] Another instance is already running; exiting duplicate process.\n", stderr)
            NSApplication.shared.terminate(nil)
            exit(0)
        }

        if AppRuntimeEnvironment.needsManualActivationPolicy {
            NSApplication.shared.setActivationPolicy(.regular)
            NSApplication.shared.activate(ignoringOtherApps: true)
        }
    }

    private func acquireSingleInstanceLock() -> Bool {
        if lockFD != nil {
            return true
        }

        let lockPath = FileManager.default.temporaryDirectory
            .appendingPathComponent("beluna.apple-universal.instance.lock")
            .path

        let fd = open(lockPath, O_CREAT | O_RDWR, S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH)
        guard fd >= 0 else {
            return true
        }

        var lock = flock()
        lock.l_type = Int16(F_WRLCK)
        lock.l_whence = Int16(SEEK_SET)
        lock.l_start = 0
        lock.l_len = 0
        lock.l_pid = 0
        if fcntl(fd, F_SETLK, &lock) == -1 {
            close(fd)
            return false
        }

        lockFD = fd
        return true
    }
}
