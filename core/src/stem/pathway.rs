#[derive(Debug, Clone, PartialEq)]
pub enum ContinueOutput<S> {
    Original,
    Replace(Vec<S>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathwayMiddlewareDecision<S, A> {
    Accepted(A),
    Rejected {
        reason_code: String,
        message: Option<String>,
    },
    Continue(ContinueOutput<S>),
}
