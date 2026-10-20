//! Run-time feature detection on PowerPC.

features! {
    @TARGET: powerpc;
    @CFG: any(target_arch = "powerpc", target_arch = "powerpc64");
    @MACRO_NAME: is_powerpc_feature_detected;
    @MACRO_ATTRS:
    /// Check for the presence of a CPU feature at runtime.
    ///
    /// When the feature is known to be enabled at compile time (e.g. via `-Ctarget-feature`)
    /// the macro expands to `true`.
    #[unstable(feature = "stdarch_powerpc_feature_detection", issue = "111191")]
    @FEATURE: #[unstable(feature = "stdarch_powerpc_feature_detection", issue = "111191")] altivec: "altivec";
    /// Altivec
    @FEATURE: #[unstable(feature = "stdarch_powerpc_feature_detection", issue = "111191")] vsx: "vsx";
    /// VSX
    @FEATURE: #[unstable(feature = "stdarch_powerpc_feature_detection", issue = "111191")] isa_2_06: "isa-2.06";
    /// ISA 2.06 (power7)
    @FEATURE: #[unstable(feature = "stdarch_powerpc_feature_detection", issue = "111191")] isa_2_07: "isa-2.07";
    /// ISA 2.07 (power8)
    @FEATURE: #[unstable(feature = "stdarch_powerpc_feature_detection", issue = "111191")] isa_3_0: "isa-3.0";
    /// ISA 3,0 (power9)
    @FEATURE: #[unstable(feature = "stdarch_powerpc_feature_detection", issue = "111191")] isa_3_1: "isa-3.1";
    /// ISA 3.1 (power10)
    @FEATURE: #[unstable(feature = "stdarch_powerpc_feature_detection", issue = "111191")] crypto: "crypto";
    /// Crypto
}
