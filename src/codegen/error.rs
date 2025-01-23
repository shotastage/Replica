use std::fmt;
use thiserror::Error;

/// Result type alias for code generation operations
pub type CodeGenResult<T> = Result<T, CodeGenError>;

/// Errors that can occur during code generation
#[derive(Error, Debug)]
pub enum CodeGenError {
    /// Error during type conversion
    #[error("Type conversion error: {0}")]
    TypeConversion(String),

    /// Error during expression compilation
    #[error("Expression compilation error: {0}")]
    ExpressionCompilation(String),

    /// Error during method compilation
    #[error("Method compilation error: {0}")]
    MethodCompilation(String),

    /// Error during WASM generation
    #[error("WASM generation error: {0}")]
    WasmGen(String),

    /// Error during module validation
    #[error("Validation error: {0}")]
    Validation(String),

    /// Error during initialization
    #[error("Initialization error: {0}")]
    Initialization(String),

    /// Error for undefined variables
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    /// Error for invalid operations
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Error for ownership violations
    #[error("Ownership error: {0}")]
    OwnershipViolation(String),

    /// Error for async/await related issues
    #[error("Async error: {0}")]
    AsyncError(String),

    /// Error for memory management issues
    #[error("Memory management error: {0}")]
    MemoryError(String),

    /// Error for LLVM-specific issues
    #[error("LLVM error: {0}")]
    LLVMError(String),

    /// Internal compiler error
    #[error("Internal compiler error: {0}")]
    Internal(String),
}

/// Detailed error information for debugging
#[derive(Debug)]
pub struct ErrorContext {
    /// Source location where the error occurred
    pub location: Option<SourceLocation>,
    /// Additional context about the error
    pub context: String,
    /// Suggestion for fixing the error
    pub suggestion: Option<String>,
}

/// Source code location information
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

impl CodeGenError {
    /// Creates a new error with additional context
    pub fn with_context(self, context: ErrorContext) -> Self {
        // 将来的にエラーコンテキストを保持する機能を追加可能
        self
    }

    /// Adds a suggestion to the error
    pub fn with_suggestion(self, suggestion: String) -> Self {
        // 将来的に提案を保持する機能を追加可能
        self
    }

    /// Returns true if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            CodeGenError::TypeConversion(_)
            | CodeGenError::ExpressionCompilation(_)
            | CodeGenError::MethodCompilation(_) => true,
            CodeGenError::Internal(_) | CodeGenError::LLVMError(_) => false,
            _ => true,
        }
    }

    /// Returns the error category
    pub fn category(&self) -> ErrorCategory {
        match self {
            CodeGenError::TypeConversion(_) => ErrorCategory::Type,
            CodeGenError::ExpressionCompilation(_) => ErrorCategory::Expression,
            CodeGenError::MethodCompilation(_) => ErrorCategory::Method,
            CodeGenError::WasmGen(_) => ErrorCategory::Wasm,
            CodeGenError::Validation(_) => ErrorCategory::Validation,
            CodeGenError::Initialization(_) => ErrorCategory::Initialization,
            CodeGenError::UndefinedVariable(_) => ErrorCategory::Variable,
            CodeGenError::InvalidOperation(_) => ErrorCategory::Operation,
            CodeGenError::OwnershipViolation(_) => ErrorCategory::Ownership,
            CodeGenError::AsyncError(_) => ErrorCategory::Async,
            CodeGenError::MemoryError(_) => ErrorCategory::Memory,
            CodeGenError::LLVMError(_) => ErrorCategory::LLVM,
            CodeGenError::Internal(_) => ErrorCategory::Internal,
        }
    }
}

/// Categories of errors for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Type,
    Expression,
    Method,
    Wasm,
    Validation,
    Initialization,
    Variable,
    Operation,
    Ownership,
    Async,
    Memory,
    LLVM,
    Internal,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCategory::Type => write!(f, "Type Error"),
            ErrorCategory::Expression => write!(f, "Expression Error"),
            ErrorCategory::Method => write!(f, "Method Error"),
            ErrorCategory::Wasm => write!(f, "WASM Generation Error"),
            ErrorCategory::Validation => write!(f, "Validation Error"),
            ErrorCategory::Initialization => write!(f, "Initialization Error"),
            ErrorCategory::Variable => write!(f, "Variable Error"),
            ErrorCategory::Operation => write!(f, "Operation Error"),
            ErrorCategory::Ownership => write!(f, "Ownership Error"),
            ErrorCategory::Async => write!(f, "Async Error"),
            ErrorCategory::Memory => write!(f, "Memory Error"),
            ErrorCategory::LLVM => write!(f, "LLVM Error"),
            ErrorCategory::Internal => write!(f, "Internal Error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = CodeGenError::TypeConversion("Invalid type".to_string());
        assert!(matches!(error, CodeGenError::TypeConversion(_)));
    }

    #[test]
    fn test_error_context() {
        let location = SourceLocation {
            file: "test.rs".to_string(),
            line: 1,
            column: 1,
        };
        let context = ErrorContext {
            location: Some(location),
            context: "Test context".to_string(),
            suggestion: Some("Try this".to_string()),
        };
        let error = CodeGenError::TypeConversion("test".to_string()).with_context(context);
        assert!(matches!(error, CodeGenError::TypeConversion(_)));
    }

    #[test]
    fn test_error_category() {
        let error = CodeGenError::TypeConversion("test".to_string());
        assert_eq!(error.category(), ErrorCategory::Type);
    }

    #[test]
    fn test_error_recoverability() {
        let recoverable = CodeGenError::TypeConversion("test".to_string());
        let unrecoverable = CodeGenError::Internal("test".to_string());
        assert!(recoverable.is_recoverable());
        assert!(!unrecoverable.is_recoverable());
    }

    #[test]
    fn test_source_location_display() {
        let location = SourceLocation {
            file: "test.rs".to_string(),
            line: 42,
            column: 10,
        };
        assert_eq!(location.to_string(), "test.rs:42:10");
    }

    #[test]
    fn test_error_category_display() {
        assert_eq!(ErrorCategory::Type.to_string(), "Type Error");
        assert_eq!(ErrorCategory::Expression.to_string(), "Expression Error");
        assert_eq!(ErrorCategory::Method.to_string(), "Method Error");
    }
}
