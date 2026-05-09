//
//  BelunaAppApp.swift
//  BelunaApp
//
//  Created by Lan_zhijiang on 2026/2/20.
//

import SwiftUI

@main
struct BelunaAppApp: App {
    @StateObject private var viewModel = ChatViewModel()
    @StateObject private var moiraViewModel: MoiraOperationsViewModel
    @StateObject private var moiraO11yViewModel: MoiraO11yViewModel

    init() {
        let moiraClient = MoiraRuntimeClientFactory.makeDefault()
        _moiraViewModel = StateObject(
            wrappedValue: MoiraOperationsViewModel(client: moiraClient)
        )
        _moiraO11yViewModel = StateObject(
            wrappedValue: MoiraO11yViewModel(client: moiraClient)
        )
    }

    var body: some Scene {
        Window("Beluna", id: "main") {
            ChatView(viewModel: viewModel)
        }
        .windowStyle(.hiddenTitleBar) // macOS 特有的 Window 设置
        .commands {
            SidebarCommands() // macOS 特有的菜单栏命令
        }

        Window("Core Control", id: "core-control") {
            MoiraCoreControlPanel(viewModel: moiraViewModel)
        }
        .defaultSize(width: 640, height: 560)

        Window("O11y / Lachesis", id: "o11y-lachesis") {
            MoiraO11yPanel(viewModel: moiraO11yViewModel)
        }
        .defaultSize(width: 980, height: 680)

        Settings {
            SettingView(viewModel: viewModel, moiraViewModel: moiraViewModel)
        }
    }
}
