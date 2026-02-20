//
//  BelunaAppApp.swift
//  BelunaApp
//
//  Created by Lan_zhijiang on 2026/2/20.
//

import SwiftUI
import AppKit

@main
struct BelunaAppApp: App {
    init() {
        AppRuntimeGuard.shared.bootstrapOrTerminate()
    }

    @StateObject private var viewModel = ChatViewModel()

    var body: some Scene {
        Window("Beluna", id: "main") {
            ChatView(viewModel: viewModel)
        }
        .windowStyle(.hiddenTitleBar) // macOS 特有的 Window 设置
        .commands {
            SidebarCommands() // macOS 特有的菜单栏命令
        }

        Settings {
            SettingView(viewModel: viewModel)
        }
    }
}

