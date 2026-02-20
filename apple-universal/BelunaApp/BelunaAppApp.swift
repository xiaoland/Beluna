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
    @StateObject private var observabilityViewModel = ObservabilityViewModel()

    var body: some Scene {
        Window("Beluna", id: "main") {
            ChatView(viewModel: viewModel)
        }
        .windowStyle(.hiddenTitleBar) // macOS 特有的 Window 设置
        .commands {
            SidebarCommands() // macOS 特有的菜单栏命令
        }

        Window("Observability", id: "observability") {
            ObservabilityView(viewModel: observabilityViewModel)
        }
        .windowStyle(.titleBar)

        Settings {
            SettingView(viewModel: viewModel)
        }
    }
}
