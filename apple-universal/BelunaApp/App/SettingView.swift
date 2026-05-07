import SwiftUI

struct SettingView: View {
    @ObservedObject var viewModel: ChatViewModel

    var body: some View {
        Form {
            ConnectionSettingsSection(viewModel: viewModel)
            ChatRetentionSettingsSection(viewModel: viewModel)
            RuntimeStatusSection(viewModel: viewModel)
            MoiraOperationsSection()
        }
        .formStyle(.grouped)
        .padding(16)
        .frame(minWidth: 560, minHeight: 360)
    }
}
