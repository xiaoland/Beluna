import SwiftUI

struct SettingView: View {
    @ObservedObject var viewModel: ChatViewModel
    @ObservedObject var moiraViewModel: MoiraOperationsViewModel

    var body: some View {
        Form {
            ConnectionSettingsSection(viewModel: viewModel)
            ChatRetentionSettingsSection(viewModel: viewModel)
            RuntimeStatusSection(viewModel: viewModel)
            MoiraOperationsSection(viewModel: moiraViewModel)
        }
        .formStyle(.grouped)
        .padding(16)
        .frame(minWidth: 560, minHeight: 360)
    }
}
