import SwiftUI

@main
struct BelunaAppleUniversalApp: App {
    @StateObject private var viewModel = ChatViewModel()

    var body: some Scene {
        WindowGroup("Beluna") {
            ChatView(viewModel: viewModel)
        }
    }
}
