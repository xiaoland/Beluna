import SwiftUI

struct MoiraProfileEnvironmentFilesEditor: View {
    @Binding var envFiles: [MoiraProfileEnvFileDraft]
    let isDisabled: Bool
    private let rowControlHeight: CGFloat = 22

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text("Environment Files")
                    .font(.caption.weight(.semibold))
                Spacer()
                Button(action: addEnvironmentFile) {
                    Label("Add", systemImage: "plus.circle")
                }
                .buttonStyle(.bordered)
                .disabled(isDisabled)
            }

            ForEach(Array(envFiles.enumerated()), id: \.offset) { index, envFile in
                HStack(spacing: 8) {
                    TextField("Path", text: envFilePathBinding(index: index))
                        .textFieldStyle(.roundedBorder)

                    Toggle("Required", isOn: envFileRequiredBinding(index: index))
                        .toggleStyle(.checkbox)

                    Button(role: .destructive) {
                        removeEnvironmentFile(index: index)
                    } label: {
                        Image(systemName: "trash")
                            .frame(width: rowControlHeight, height: rowControlHeight)
                    }
                    .buttonStyle(.borderless)
                }
                .id("\(index)-\(envFile.required)")
                .disabled(isDisabled)
            }
        }
    }

    private func envFilePathBinding(index: Int) -> Binding<String> {
        Binding(
            get: { envFiles[safe: index]?.path ?? "" },
            set: { value in
                guard envFiles.indices.contains(index) else {
                    return
                }
                envFiles[index].path = value
            }
        )
    }

    private func envFileRequiredBinding(index: Int) -> Binding<Bool> {
        Binding(
            get: { envFiles[safe: index]?.required ?? true },
            set: { value in
                guard envFiles.indices.contains(index) else {
                    return
                }
                envFiles[index].required = value
            }
        )
    }

    private func addEnvironmentFile() {
        envFiles.append(MoiraProfileEnvFileDraft(path: "", required: true))
    }

    private func removeEnvironmentFile(index: Int) {
        guard envFiles.indices.contains(index) else {
            return
        }
        envFiles.remove(at: index)
    }
}

struct MoiraProfileInlineEnvironmentEditor: View {
    @Binding var inlineEnvironment: [MoiraProfileInlineEnvironmentDraft]
    let isDisabled: Bool
    private let nameFieldWidth: CGFloat = 180
    private let valueFieldWidth: CGFloat = 280
    private let rowControlHeight: CGFloat = 22

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text("Inline Environment")
                    .font(.caption.weight(.semibold))
                Spacer()
                Button(action: addInlineEnvironment) {
                    Label("Add", systemImage: "plus.circle")
                }
                .buttonStyle(.bordered)
                .disabled(isDisabled)
            }

            ForEach(Array(inlineEnvironment.enumerated()), id: \.offset) { index, entry in
                HStack(alignment: .bottom, spacing: 8) {
                    inlineEnvironmentField(
                        title: "Name",
                        width: nameFieldWidth,
                        text: inlineEnvironmentKeyBinding(index: index)
                    )

                    inlineEnvironmentField(
                        title: "Value",
                        width: valueFieldWidth,
                        text: inlineEnvironmentValueBinding(index: index)
                    )

                    Button(role: .destructive) {
                        removeInlineEnvironment(index: index)
                    } label: {
                        Image(systemName: "trash")
                            .frame(width: rowControlHeight, height: rowControlHeight)
                    }
                    .buttonStyle(.borderless)
                }
                .id("\(index)-\(entry.key)")
                .disabled(isDisabled)
            }
        }
    }

    private func inlineEnvironmentField(
        title: String,
        width: CGFloat,
        text: Binding<String>
    ) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title)
                .font(.caption)
                .foregroundStyle(.secondary)

            TextField(title, text: text)
                .labelsHidden()
                .textFieldStyle(.roundedBorder)
                .frame(width: width, height: rowControlHeight)
        }
    }

    private func inlineEnvironmentKeyBinding(index: Int) -> Binding<String> {
        Binding(
            get: { inlineEnvironment[safe: index]?.key ?? "" },
            set: { value in
                guard inlineEnvironment.indices.contains(index) else {
                    return
                }
                inlineEnvironment[index].key = value
            }
        )
    }

    private func inlineEnvironmentValueBinding(index: Int) -> Binding<String> {
        Binding(
            get: { inlineEnvironment[safe: index]?.value ?? "" },
            set: { value in
                guard inlineEnvironment.indices.contains(index) else {
                    return
                }
                inlineEnvironment[index].value = value
            }
        )
    }

    private func addInlineEnvironment() {
        inlineEnvironment.append(MoiraProfileInlineEnvironmentDraft(key: "", value: ""))
    }

    private func removeInlineEnvironment(index: Int) {
        guard inlineEnvironment.indices.contains(index) else {
            return
        }
        inlineEnvironment.remove(at: index)
    }
}
