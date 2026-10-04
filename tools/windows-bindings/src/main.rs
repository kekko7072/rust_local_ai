//! Generates the Phi Silica (Windows AI `LanguageModel`) bindings.
//!
//! Only `windows-core` is referenced by the output: the Windows.Foundation
//! async types are generated too, so `windows-future` is not needed.

use std::{env, path::Path};

fn main() {
    let metadata = env::args()
        .nth(1)
        .expect("usage: windows-bindings-generator <path to the NuGet package's metadata dir>");
    let winmd = |name: &str| Path::new(&metadata).join(name).display().to_string();
    let out = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../src/backend/windows/bindings.rs")
        .display()
        .to_string();
    let inputs = [
        winmd("Microsoft.Windows.AI.Text.winmd"),
        winmd("Microsoft.Windows.AI.winmd"),
        winmd("Microsoft.Windows.AI.Foundation.winmd"),
        winmd("Microsoft.Windows.Workloads.winmd"),
        winmd("Microsoft.Graphics.Imaging.winmd"),
        winmd("Microsoft.Windows.AI.ContentSafety.winmd"),
    ];
    let mut args = vec!["--in", "default"];
    args.extend(inputs.iter().map(String::as_str));
    args.extend([
        "--out",
        &out,
        "--no-comment",
        "--filter",
        "Microsoft.Windows.AI.Text.LanguageModel",
        "Microsoft.Windows.AI.Text.LanguageModelOptions",
        "Microsoft.Windows.AI.Text.LanguageModelResponseResult",
        "Microsoft.Windows.AI.Text.LanguageModelResponseStatus",
        "Microsoft.Windows.AI.AIFeatureReadyState",
        "Microsoft.Windows.AI.AIFeatureReadyResult",
        "Microsoft.Windows.AI.AIFeatureReadyResultState",
        "Windows.Foundation.IAsyncInfo",
        "Windows.Foundation.AsyncStatus",
        "Windows.Foundation.IAsyncOperation",
        "Windows.Foundation.IAsyncOperationWithProgress",
        "Windows.Foundation.AsyncOperationCompletedHandler",
        "Windows.Foundation.AsyncOperationProgressHandler",
        "Windows.Foundation.AsyncOperationWithProgressCompletedHandler",
    ]);
    // Warnings list methods skipped for types outside the filter (embeddings,
    // LoRA adapters, content-filter options); none are used.
    let warnings = windows_bindgen::bindgen(args);
    eprintln!("{warnings}");
}
