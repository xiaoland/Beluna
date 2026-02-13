import SwiftUI
import AppKit

@main
struct BelunaApp: App {
    init() {
        // `swift run` launches a plain executable (not an .app bundle).
        // Force regular app activation so WindowGroup becomes visible.
        NSApplication.shared.setActivationPolicy(.regular)
        NSApplication.shared.activate(ignoringOtherApps: true)
    }

    @StateObject private var viewModel = ChatViewModel()

    var body: some Scene {
        WindowGroup("Beluna") {
            ChatView(viewModel: viewModel)
        }
        #if os(macOS)
        .windowStyle(.hiddenTitleBar) // macOS 特有的 Window 设置
        .commands {
            SidebarCommands() // macOS 特有的菜单栏命令
        }
        #endif
    }
}
