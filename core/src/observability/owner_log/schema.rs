#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OwnerScope {
    Main,
    Stem,
    Cortex,
    AiGateway,
    AiGatewayChat,
    Spine,
}

impl OwnerScope {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            OwnerScope::Main => "beluna.core.main",
            OwnerScope::Stem => "beluna.core.stem",
            OwnerScope::Cortex => "beluna.core.cortex",
            OwnerScope::AiGateway => "beluna.core.ai-gateway",
            OwnerScope::AiGatewayChat => "beluna.core.ai-gateway.chat",
            OwnerScope::Spine => "beluna.core.spine",
        }
    }
}
