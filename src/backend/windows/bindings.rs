#![allow(
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    dead_code,
    clippy::all
)]

pub mod Microsoft {
    pub mod Windows {
        pub mod AI {
            #[repr(transparent)]
            #[derive(Clone, Debug, Eq, PartialEq)]
            pub struct AIFeatureReadyResult(windows_core::IUnknown);
            windows_core::imp::interface_hierarchy!(
                AIFeatureReadyResult,
                windows_core::IUnknown,
                windows_core::IInspectable
            );
            impl AIFeatureReadyResult {
                pub fn Error(&self) -> windows_core::Result<windows_core::HRESULT> {
                    let this = self;
                    unsafe {
                        let mut result__ = core::mem::zeroed();
                        (windows_core::Interface::vtable(this).Error)(
                            windows_core::Interface::as_raw(this),
                            &mut result__,
                        )
                        .map(|| result__)
                    }
                }
                pub fn ErrorDisplayText(&self) -> windows_core::Result<windows_core::HSTRING> {
                    let this = self;
                    unsafe {
                        let mut result__ = core::mem::zeroed();
                        (windows_core::Interface::vtable(this).ErrorDisplayText)(
                            windows_core::Interface::as_raw(this),
                            &mut result__,
                        )
                        .map(|| core::mem::transmute(result__))
                    }
                }
                pub fn ExtendedError(&self) -> windows_core::Result<windows_core::HRESULT> {
                    let this = self;
                    unsafe {
                        let mut result__ = core::mem::zeroed();
                        (windows_core::Interface::vtable(this).ExtendedError)(
                            windows_core::Interface::as_raw(this),
                            &mut result__,
                        )
                        .map(|| result__)
                    }
                }
                pub fn Status(&self) -> windows_core::Result<AIFeatureReadyResultState> {
                    let this = self;
                    unsafe {
                        let mut result__ = core::mem::zeroed();
                        (windows_core::Interface::vtable(this).Status)(
                            windows_core::Interface::as_raw(this),
                            &mut result__,
                        )
                        .map(|| result__)
                    }
                }
                pub fn PackageInstallationFailed(&self) -> windows_core::Result<bool> {
                    let this = &windows_core::Interface::cast::<IAIFeatureReadyResult2>(self)?;
                    unsafe {
                        let mut result__ = core::mem::zeroed();
                        (windows_core::Interface::vtable(this).PackageInstallationFailed)(
                            windows_core::Interface::as_raw(this),
                            &mut result__,
                        )
                        .map(|| result__)
                    }
                }
            }
            impl windows_core::RuntimeType for AIFeatureReadyResult {
                const SIGNATURE: windows_core::imp::ConstBuffer =
                    windows_core::imp::ConstBuffer::for_class::<Self, IAIFeatureReadyResult>();
            }
            unsafe impl windows_core::Interface for AIFeatureReadyResult {
                type Vtable = <IAIFeatureReadyResult as windows_core::Interface>::Vtable;
                const IID: windows_core::GUID =
                    <IAIFeatureReadyResult as windows_core::Interface>::IID;
            }
            impl windows_core::RuntimeName for AIFeatureReadyResult {
                const NAME: &'static str = "Microsoft.Windows.AI.AIFeatureReadyResult";
            }
            unsafe impl Send for AIFeatureReadyResult {}
            unsafe impl Sync for AIFeatureReadyResult {}
            #[repr(transparent)]
            #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
            pub struct AIFeatureReadyResultState(pub i32);
            impl AIFeatureReadyResultState {
                pub const InProgress: Self = Self(0i32);
                pub const Success: Self = Self(1i32);
                pub const Failure: Self = Self(2i32);
            }
            impl windows_core::TypeKind for AIFeatureReadyResultState {
                type TypeKind = windows_core::CopyType;
            }
            impl windows_core::RuntimeType for AIFeatureReadyResultState {
                const SIGNATURE: windows_core::imp::ConstBuffer =
                    windows_core::imp::ConstBuffer::from_slice(
                        b"enum(Microsoft.Windows.AI.AIFeatureReadyResultState;i4)",
                    );
            }
            #[repr(transparent)]
            #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
            pub struct AIFeatureReadyState(pub i32);
            impl AIFeatureReadyState {
                pub const Ready: Self = Self(0i32);
                pub const NotReady: Self = Self(1i32);
                pub const NotSupportedOnCurrentSystem: Self = Self(2i32);
                pub const DisabledByUser: Self = Self(3i32);
                pub const CapabilityMissing: Self = Self(4i32);
                pub const NotCompatibleWithSystemHardware: Self = Self(5i32);
                pub const OSUpdateNeeded: Self = Self(6i32);
            }
            impl windows_core::TypeKind for AIFeatureReadyState {
                type TypeKind = windows_core::CopyType;
            }
            impl windows_core::RuntimeType for AIFeatureReadyState {
                const SIGNATURE: windows_core::imp::ConstBuffer =
                    windows_core::imp::ConstBuffer::from_slice(
                        b"enum(Microsoft.Windows.AI.AIFeatureReadyState;i4)",
                    );
            }
            windows_core::imp::define_interface!(
                IAIFeatureReadyResult,
                IAIFeatureReadyResult_Vtbl,
                0x936a78a6_c242_5937_9814_e512d4193a6d
            );
            impl windows_core::RuntimeType for IAIFeatureReadyResult {
                const SIGNATURE: windows_core::imp::ConstBuffer =
                    windows_core::imp::ConstBuffer::for_interface::<Self>();
            }
            #[repr(C)]
            #[doc(hidden)]
            pub struct IAIFeatureReadyResult_Vtbl {
                pub base__: windows_core::IInspectable_Vtbl,
                pub Error: unsafe extern "system" fn(
                    *mut core::ffi::c_void,
                    *mut windows_core::HRESULT,
                ) -> windows_core::HRESULT,
                pub ErrorDisplayText: unsafe extern "system" fn(
                    *mut core::ffi::c_void,
                    *mut *mut core::ffi::c_void,
                )
                    -> windows_core::HRESULT,
                pub ExtendedError: unsafe extern "system" fn(
                    *mut core::ffi::c_void,
                    *mut windows_core::HRESULT,
                )
                    -> windows_core::HRESULT,
                pub Status: unsafe extern "system" fn(
                    *mut core::ffi::c_void,
                    *mut AIFeatureReadyResultState,
                ) -> windows_core::HRESULT,
            }
            windows_core::imp::define_interface!(
                IAIFeatureReadyResult2,
                IAIFeatureReadyResult2_Vtbl,
                0xec5f1d67_43c1_5bdb_b9f4_a0c7110582cb
            );
            impl windows_core::RuntimeType for IAIFeatureReadyResult2 {
                const SIGNATURE: windows_core::imp::ConstBuffer =
                    windows_core::imp::ConstBuffer::for_interface::<Self>();
            }
            #[repr(C)]
            #[doc(hidden)]
            pub struct IAIFeatureReadyResult2_Vtbl {
                pub base__: windows_core::IInspectable_Vtbl,
                pub PackageInstallationFailed: unsafe extern "system" fn(
                    *mut core::ffi::c_void,
                    *mut bool,
                )
                    -> windows_core::HRESULT,
            }
            pub mod Text {
                windows_core::imp::define_interface!(
                    ILanguageModel,
                    ILanguageModel_Vtbl,
                    0x6331c629_8c86_5bfe_8c4e_9ca5573cc14b
                );
                impl windows_core::RuntimeType for ILanguageModel {
                    const SIGNATURE: windows_core::imp::ConstBuffer =
                        windows_core::imp::ConstBuffer::for_interface::<Self>();
                }
                #[repr(C)]
                #[doc(hidden)]
                pub struct ILanguageModel_Vtbl {
                    pub base__: windows_core::IInspectable_Vtbl,
                }
                windows_core::imp::define_interface!(
                    ILanguageModel2,
                    ILanguageModel2_Vtbl,
                    0x653b714e_f9b3_51cb_954f_5ea58f63ab89
                );
                impl windows_core::RuntimeType for ILanguageModel2 {
                    const SIGNATURE: windows_core::imp::ConstBuffer =
                        windows_core::imp::ConstBuffer::for_interface::<Self>();
                }
                #[repr(C)]
                #[doc(hidden)]
                pub struct ILanguageModel2_Vtbl {
                    pub base__: windows_core::IInspectable_Vtbl,
                    pub GenerateResponseAsync: unsafe extern "system" fn(
                        *mut core::ffi::c_void,
                        *mut core::ffi::c_void,
                        *mut *mut core::ffi::c_void,
                    )
                        -> windows_core::HRESULT,
                    pub GenerateResponseAsync2: unsafe extern "system" fn(
                        *mut core::ffi::c_void,
                        *mut core::ffi::c_void,
                        *mut core::ffi::c_void,
                        *mut *mut core::ffi::c_void,
                    )
                        -> windows_core::HRESULT,
                    GenerateResponseAsync3: usize,
                    GenerateResponseFromEmbeddingsAsync: usize,
                    GenerateResponseFromEmbeddingsAsync2: usize,
                    GenerateResponseFromEmbeddingsAsync3: usize,
                    GenerateEmbeddingVectors: usize,
                    GenerateEmbeddingVectors2: usize,
                    pub GetUsablePromptLength: unsafe extern "system" fn(
                        *mut core::ffi::c_void,
                        *mut core::ffi::c_void,
                        *mut u64,
                    )
                        -> windows_core::HRESULT,
                    GetUsablePromptLength2: usize,
                    pub GetVectorSpaceId: unsafe extern "system" fn(
                        *mut core::ffi::c_void,
                        *mut windows_core::GUID,
                    )
                        -> windows_core::HRESULT,
                    CreateContext: usize,
                    CreateContext2: usize,
                    CreateContext3: usize,
                }
                windows_core::imp::define_interface!(
                    ILanguageModel3,
                    ILanguageModel3_Vtbl,
                    0xe5f6b00d_a835_5170_864c_4aa810a7fd15
                );
                impl windows_core::RuntimeType for ILanguageModel3 {
                    const SIGNATURE: windows_core::imp::ConstBuffer =
                        windows_core::imp::ConstBuffer::for_interface::<Self>();
                }
                #[repr(C)]
                #[doc(hidden)]
                pub struct ILanguageModel3_Vtbl {
                    pub base__: windows_core::IInspectable_Vtbl,
                    GenerateStructuredJsonResponseAsync: usize,
                    GenerateStructuredJsonResponseAsync2: usize,
                }
                windows_core::imp::define_interface!(
                    ILanguageModelOptions,
                    ILanguageModelOptions_Vtbl,
                    0x7f380003_5a09_5f1f_afb0_aa483e3670cc
                );
                impl windows_core::RuntimeType for ILanguageModelOptions {
                    const SIGNATURE: windows_core::imp::ConstBuffer =
                        windows_core::imp::ConstBuffer::for_interface::<Self>();
                }
                #[repr(C)]
                #[doc(hidden)]
                pub struct ILanguageModelOptions_Vtbl {
                    pub base__: windows_core::IInspectable_Vtbl,
                    pub Temperature: unsafe extern "system" fn(
                        *mut core::ffi::c_void,
                        *mut f32,
                    )
                        -> windows_core::HRESULT,
                    pub SetTemperature: unsafe extern "system" fn(
                        *mut core::ffi::c_void,
                        f32,
                    )
                        -> windows_core::HRESULT,
                    pub TopP: unsafe extern "system" fn(
                        *mut core::ffi::c_void,
                        *mut f32,
                    )
                        -> windows_core::HRESULT,
                    pub SetTopP: unsafe extern "system" fn(
                        *mut core::ffi::c_void,
                        f32,
                    )
                        -> windows_core::HRESULT,
                    pub TopK: unsafe extern "system" fn(
                        *mut core::ffi::c_void,
                        *mut u32,
                    )
                        -> windows_core::HRESULT,
                    pub SetTopK: unsafe extern "system" fn(
                        *mut core::ffi::c_void,
                        u32,
                    )
                        -> windows_core::HRESULT,
                    ContentFilterOptions: usize,
                    SetContentFilterOptions: usize,
                }
                windows_core::imp::define_interface!(
                    ILanguageModelOptions2,
                    ILanguageModelOptions2_Vtbl,
                    0x8fac9616_a95f_56f7_96d8_e01ebdda1d39
                );
                impl windows_core::RuntimeType for ILanguageModelOptions2 {
                    const SIGNATURE: windows_core::imp::ConstBuffer =
                        windows_core::imp::ConstBuffer::for_interface::<Self>();
                }
                #[repr(C)]
                #[doc(hidden)]
                pub struct ILanguageModelOptions2_Vtbl {
                    pub base__: windows_core::IInspectable_Vtbl,
                    LowRankAdapter: usize,
                    SetLowRankAdapter: usize,
                }
                windows_core::imp::define_interface!(
                    ILanguageModelResponseResult,
                    ILanguageModelResponseResult_Vtbl,
                    0x3a256fff_a426_5d3b_8e4b_3ac84162471e
                );
                impl windows_core::RuntimeType for ILanguageModelResponseResult {
                    const SIGNATURE: windows_core::imp::ConstBuffer =
                        windows_core::imp::ConstBuffer::for_interface::<Self>();
                }
                #[repr(C)]
                #[doc(hidden)]
                pub struct ILanguageModelResponseResult_Vtbl {
                    pub base__: windows_core::IInspectable_Vtbl,
                    pub Text: unsafe extern "system" fn(
                        *mut core::ffi::c_void,
                        *mut *mut core::ffi::c_void,
                    )
                        -> windows_core::HRESULT,
                    pub Status: unsafe extern "system" fn(
                        *mut core::ffi::c_void,
                        *mut LanguageModelResponseStatus,
                    )
                        -> windows_core::HRESULT,
                    pub ExtendedError: unsafe extern "system" fn(
                        *mut core::ffi::c_void,
                        *mut windows_core::HRESULT,
                    )
                        -> windows_core::HRESULT,
                }
                windows_core::imp::define_interface!(
                    ILanguageModelStatics,
                    ILanguageModelStatics_Vtbl,
                    0x8f18f9af_6095_553b_8d9d_6bcc98026546
                );
                impl windows_core::RuntimeType for ILanguageModelStatics {
                    const SIGNATURE: windows_core::imp::ConstBuffer =
                        windows_core::imp::ConstBuffer::for_interface::<Self>();
                }
                #[repr(C)]
                #[doc(hidden)]
                pub struct ILanguageModelStatics_Vtbl {
                    pub base__: windows_core::IInspectable_Vtbl,
                    pub GetReadyState: unsafe extern "system" fn(
                        *mut core::ffi::c_void,
                        *mut super::AIFeatureReadyState,
                    )
                        -> windows_core::HRESULT,
                    pub EnsureReadyAsync: unsafe extern "system" fn(
                        *mut core::ffi::c_void,
                        *mut *mut core::ffi::c_void,
                    )
                        -> windows_core::HRESULT,
                    pub CreateAsync: unsafe extern "system" fn(
                        *mut core::ffi::c_void,
                        *mut *mut core::ffi::c_void,
                    )
                        -> windows_core::HRESULT,
                }
                #[repr(transparent)]
                #[derive(Clone, Debug, Eq, PartialEq)]
                pub struct LanguageModel(windows_core::IUnknown);
                windows_core::imp::interface_hierarchy!(
                    LanguageModel,
                    windows_core::IUnknown,
                    windows_core::IInspectable
                );
                windows_core::imp::required_hierarchy!(
                    LanguageModel,
                    super::super::super::super::Windows::Foundation::IClosable
                );
                impl LanguageModel {
                    pub fn Close(&self) -> windows_core::Result<()> {
                        let this = &windows_core::Interface::cast::<
                            super::super::super::super::Windows::Foundation::IClosable,
                        >(self)?;
                        unsafe {
                            (windows_core::Interface::vtable(this).Close)(
                                windows_core::Interface::as_raw(this),
                            )
                            .ok()
                        }
                    }                    pub fn GenerateResponseAsync < > ( & self , prompt : & windows_core::HSTRING , ) -> windows_core::Result < super::super::super::super::Windows::Foundation:: IAsyncOperationWithProgress < LanguageModelResponseResult , windows_core::HSTRING > >{
                        let this = &windows_core::Interface::cast::<ILanguageModel2>(self)?;
                        unsafe {
                            let mut result__ = core::mem::zeroed();
                            (windows_core::Interface::vtable(this).GenerateResponseAsync)(
                                windows_core::Interface::as_raw(this),
                                core::mem::transmute_copy(prompt),
                                &mut result__,
                            )
                            .and_then(|| windows_core::Type::from_abi(result__))
                        }
                    }                    pub fn GenerateResponseAsync2 < P1 , > ( & self , prompt : & windows_core::HSTRING , options : P1 , ) -> windows_core::Result < super::super::super::super::Windows::Foundation:: IAsyncOperationWithProgress < LanguageModelResponseResult , windows_core::HSTRING > > where P1 :windows_core::Param < LanguageModelOptions > ,{
                        let this = &windows_core::Interface::cast::<ILanguageModel2>(self)?;
                        unsafe {
                            let mut result__ = core::mem::zeroed();
                            (windows_core::Interface::vtable(this).GenerateResponseAsync2)(
                                windows_core::Interface::as_raw(this),
                                core::mem::transmute_copy(prompt),
                                options.param().abi(),
                                &mut result__,
                            )
                            .and_then(|| windows_core::Type::from_abi(result__))
                        }
                    }
                    pub fn GetUsablePromptLength(
                        &self,
                        prompt: &windows_core::HSTRING,
                    ) -> windows_core::Result<u64> {
                        let this = &windows_core::Interface::cast::<ILanguageModel2>(self)?;
                        unsafe {
                            let mut result__ = core::mem::zeroed();
                            (windows_core::Interface::vtable(this).GetUsablePromptLength)(
                                windows_core::Interface::as_raw(this),
                                core::mem::transmute_copy(prompt),
                                &mut result__,
                            )
                            .map(|| result__)
                        }
                    }
                    pub fn GetVectorSpaceId(&self) -> windows_core::Result<windows_core::GUID> {
                        let this = &windows_core::Interface::cast::<ILanguageModel2>(self)?;
                        unsafe {
                            let mut result__ = core::mem::zeroed();
                            (windows_core::Interface::vtable(this).GetVectorSpaceId)(
                                windows_core::Interface::as_raw(this),
                                &mut result__,
                            )
                            .map(|| result__)
                        }
                    }
                    pub fn GetReadyState() -> windows_core::Result<super::AIFeatureReadyState> {
                        Self::ILanguageModelStatics(|this| unsafe {
                            let mut result__ = core::mem::zeroed();
                            (windows_core::Interface::vtable(this).GetReadyState)(
                                windows_core::Interface::as_raw(this),
                                &mut result__,
                            )
                            .map(|| result__)
                        })
                    }                    pub fn EnsureReadyAsync < > ( ) -> windows_core::Result < super::super::super::super::Windows::Foundation:: IAsyncOperationWithProgress < super:: AIFeatureReadyResult , f64 > >{
                        Self::ILanguageModelStatics(|this| unsafe {
                            let mut result__ = core::mem::zeroed();
                            (windows_core::Interface::vtable(this).EnsureReadyAsync)(
                                windows_core::Interface::as_raw(this),
                                &mut result__,
                            )
                            .and_then(|| windows_core::Type::from_abi(result__))
                        })
                    }
                    pub fn CreateAsync() -> windows_core::Result<
                        super::super::super::super::Windows::Foundation::IAsyncOperation<
                            LanguageModel,
                        >,
                    > {
                        Self::ILanguageModelStatics(|this| unsafe {
                            let mut result__ = core::mem::zeroed();
                            (windows_core::Interface::vtable(this).CreateAsync)(
                                windows_core::Interface::as_raw(this),
                                &mut result__,
                            )
                            .and_then(|| windows_core::Type::from_abi(result__))
                        })
                    }
                    fn ILanguageModelStatics<
                        R,
                        F: FnOnce(&ILanguageModelStatics) -> windows_core::Result<R>,
                    >(
                        callback: F,
                    ) -> windows_core::Result<R> {
                        static SHARED: windows_core::imp::FactoryCache<
                            LanguageModel,
                            ILanguageModelStatics,
                        > = windows_core::imp::FactoryCache::new();
                        SHARED.call(callback)
                    }
                }
                impl windows_core::RuntimeType for LanguageModel {
                    const SIGNATURE: windows_core::imp::ConstBuffer =
                        windows_core::imp::ConstBuffer::for_class::<Self, ILanguageModel>();
                }
                unsafe impl windows_core::Interface for LanguageModel {
                    type Vtable = <ILanguageModel as windows_core::Interface>::Vtable;
                    const IID: windows_core::GUID =
                        <ILanguageModel as windows_core::Interface>::IID;
                }
                impl windows_core::RuntimeName for LanguageModel {
                    const NAME: &'static str = "Microsoft.Windows.AI.Text.LanguageModel";
                }
                unsafe impl Send for LanguageModel {}
                unsafe impl Sync for LanguageModel {}
                #[repr(transparent)]
                #[derive(Clone, Debug, Eq, PartialEq)]
                pub struct LanguageModelOptions(windows_core::IUnknown);
                windows_core::imp::interface_hierarchy!(
                    LanguageModelOptions,
                    windows_core::IUnknown,
                    windows_core::IInspectable
                );
                impl LanguageModelOptions {
                    pub fn new() -> windows_core::Result<Self> {
                        Self::IActivationFactory(|f| f.ActivateInstance::<Self>())
                    }
                    fn IActivationFactory<
                        R,
                        F: FnOnce(&windows_core::imp::IGenericFactory) -> windows_core::Result<R>,
                    >(
                        callback: F,
                    ) -> windows_core::Result<R> {
                        static SHARED: windows_core::imp::FactoryCache<
                            LanguageModelOptions,
                            windows_core::imp::IGenericFactory,
                        > = windows_core::imp::FactoryCache::new();
                        SHARED.call(callback)
                    }
                    pub fn Temperature(&self) -> windows_core::Result<f32> {
                        let this = self;
                        unsafe {
                            let mut result__ = core::mem::zeroed();
                            (windows_core::Interface::vtable(this).Temperature)(
                                windows_core::Interface::as_raw(this),
                                &mut result__,
                            )
                            .map(|| result__)
                        }
                    }
                    pub fn SetTemperature(&self, value: f32) -> windows_core::Result<()> {
                        let this = self;
                        unsafe {
                            (windows_core::Interface::vtable(this).SetTemperature)(
                                windows_core::Interface::as_raw(this),
                                value,
                            )
                            .ok()
                        }
                    }
                    pub fn TopP(&self) -> windows_core::Result<f32> {
                        let this = self;
                        unsafe {
                            let mut result__ = core::mem::zeroed();
                            (windows_core::Interface::vtable(this).TopP)(
                                windows_core::Interface::as_raw(this),
                                &mut result__,
                            )
                            .map(|| result__)
                        }
                    }
                    pub fn SetTopP(&self, value: f32) -> windows_core::Result<()> {
                        let this = self;
                        unsafe {
                            (windows_core::Interface::vtable(this).SetTopP)(
                                windows_core::Interface::as_raw(this),
                                value,
                            )
                            .ok()
                        }
                    }
                    pub fn TopK(&self) -> windows_core::Result<u32> {
                        let this = self;
                        unsafe {
                            let mut result__ = core::mem::zeroed();
                            (windows_core::Interface::vtable(this).TopK)(
                                windows_core::Interface::as_raw(this),
                                &mut result__,
                            )
                            .map(|| result__)
                        }
                    }
                    pub fn SetTopK(&self, value: u32) -> windows_core::Result<()> {
                        let this = self;
                        unsafe {
                            (windows_core::Interface::vtable(this).SetTopK)(
                                windows_core::Interface::as_raw(this),
                                value,
                            )
                            .ok()
                        }
                    }
                }
                impl windows_core::RuntimeType for LanguageModelOptions {
                    const SIGNATURE: windows_core::imp::ConstBuffer =
                        windows_core::imp::ConstBuffer::for_class::<Self, ILanguageModelOptions>();
                }
                unsafe impl windows_core::Interface for LanguageModelOptions {
                    type Vtable = <ILanguageModelOptions as windows_core::Interface>::Vtable;
                    const IID: windows_core::GUID =
                        <ILanguageModelOptions as windows_core::Interface>::IID;
                }
                impl windows_core::RuntimeName for LanguageModelOptions {
                    const NAME: &'static str = "Microsoft.Windows.AI.Text.LanguageModelOptions";
                }
                unsafe impl Send for LanguageModelOptions {}
                unsafe impl Sync for LanguageModelOptions {}
                #[repr(transparent)]
                #[derive(Clone, Debug, Eq, PartialEq)]
                pub struct LanguageModelResponseResult(windows_core::IUnknown);
                windows_core::imp::interface_hierarchy!(
                    LanguageModelResponseResult,
                    windows_core::IUnknown,
                    windows_core::IInspectable
                );
                impl LanguageModelResponseResult {
                    pub fn Text(&self) -> windows_core::Result<windows_core::HSTRING> {
                        let this = self;
                        unsafe {
                            let mut result__ = core::mem::zeroed();
                            (windows_core::Interface::vtable(this).Text)(
                                windows_core::Interface::as_raw(this),
                                &mut result__,
                            )
                            .map(|| core::mem::transmute(result__))
                        }
                    }
                    pub fn Status(&self) -> windows_core::Result<LanguageModelResponseStatus> {
                        let this = self;
                        unsafe {
                            let mut result__ = core::mem::zeroed();
                            (windows_core::Interface::vtable(this).Status)(
                                windows_core::Interface::as_raw(this),
                                &mut result__,
                            )
                            .map(|| result__)
                        }
                    }
                    pub fn ExtendedError(&self) -> windows_core::Result<windows_core::HRESULT> {
                        let this = self;
                        unsafe {
                            let mut result__ = core::mem::zeroed();
                            (windows_core::Interface::vtable(this).ExtendedError)(
                                windows_core::Interface::as_raw(this),
                                &mut result__,
                            )
                            .map(|| result__)
                        }
                    }
                }
                impl windows_core::RuntimeType for LanguageModelResponseResult {
                    const SIGNATURE: windows_core::imp::ConstBuffer =
                        windows_core::imp::ConstBuffer::for_class::<
                            Self,
                            ILanguageModelResponseResult,
                        >();
                }
                unsafe impl windows_core::Interface for LanguageModelResponseResult {
                    type Vtable = <ILanguageModelResponseResult as windows_core::Interface>::Vtable;
                    const IID: windows_core::GUID =
                        <ILanguageModelResponseResult as windows_core::Interface>::IID;
                }
                impl windows_core::RuntimeName for LanguageModelResponseResult {
                    const NAME: &'static str =
                        "Microsoft.Windows.AI.Text.LanguageModelResponseResult";
                }
                unsafe impl Send for LanguageModelResponseResult {}
                unsafe impl Sync for LanguageModelResponseResult {}
                #[repr(transparent)]
                #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
                pub struct LanguageModelResponseStatus(pub i32);
                impl LanguageModelResponseStatus {
                    pub const Complete: Self = Self(0i32);
                    pub const InProgress: Self = Self(1i32);
                    pub const BlockedByPolicy: Self = Self(2i32);
                    pub const PromptLargerThanContext: Self = Self(3i32);
                    pub const PromptBlockedByContentModeration: Self = Self(4i32);
                    pub const ResponseBlockedByContentModeration: Self = Self(5i32);
                    pub const Error: Self = Self(6i32);
                    pub const IncompatibleLowRankAdapter: Self = Self(7i32);
                    pub const UnsupportedLanguage: Self = Self(8i32);
                    pub const LanguageMismatch: Self = Self(9i32);
                }
                impl windows_core::TypeKind for LanguageModelResponseStatus {
                    type TypeKind = windows_core::CopyType;
                }
                impl windows_core::RuntimeType for LanguageModelResponseStatus {
                    const SIGNATURE: windows_core::imp::ConstBuffer =
                        windows_core::imp::ConstBuffer::from_slice(
                            b"enum(Microsoft.Windows.AI.Text.LanguageModelResponseStatus;i4)",
                        );
                }
            }
        }
    }
}
pub mod Windows {
    pub mod Foundation {
        #[repr(transparent)]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct AsyncOperationCompletedHandler<TResult>(
            windows_core::IUnknown,
            core::marker::PhantomData<TResult>,
        )
        where
            TResult: windows_core::RuntimeType + 'static;
        unsafe impl<TResult: windows_core::RuntimeType + 'static> windows_core::Interface
            for AsyncOperationCompletedHandler<TResult>
        {
            type Vtable = AsyncOperationCompletedHandler_Vtbl<TResult>;
            const IID: windows_core::GUID =
                windows_core::GUID::from_signature(<Self as windows_core::RuntimeType>::SIGNATURE);
        }
        impl<TResult: windows_core::RuntimeType + 'static> windows_core::RuntimeType
            for AsyncOperationCompletedHandler<TResult>
        {
            const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::new()
                .push_slice(b"pinterface({fcdcf02c-e5d8-4478-915a-4d90b74b83a5}")
                .push_slice(b";")
                .push_other(TResult::SIGNATURE)
                .push_slice(b")");
        }
        impl<TResult: windows_core::RuntimeType + 'static> AsyncOperationCompletedHandler<TResult> {
            pub fn new<
                F: FnMut(
                        windows_core::Ref<'_, IAsyncOperation<TResult>>,
                        AsyncStatus,
                    ) -> windows_core::Result<()>
                    + Send
                    + 'static,
            >(
                invoke: F,
            ) -> Self {
                let com = AsyncOperationCompletedHandlerBox {
                    vtable: &AsyncOperationCompletedHandlerBox::<TResult, F>::VTABLE,
                    count: windows_core::imp::RefCount::new(1),
                    invoke,
                };
                unsafe { core::mem::transmute(windows_core::imp::Box::new(com)) }
            }
            pub fn Invoke<P0>(
                &self,
                asyncinfo: P0,
                asyncstatus: AsyncStatus,
            ) -> windows_core::Result<()>
            where
                P0: windows_core::Param<IAsyncOperation<TResult>>,
            {
                let this = self;
                unsafe {
                    (windows_core::Interface::vtable(this).Invoke)(
                        windows_core::Interface::as_raw(this),
                        asyncinfo.param().abi(),
                        asyncstatus,
                    )
                    .ok()
                }
            }
        }
        #[repr(C)]
        #[doc(hidden)]
        pub struct AsyncOperationCompletedHandler_Vtbl<TResult>
        where
            TResult: windows_core::RuntimeType + 'static,
        {
            base__: windows_core::IUnknown_Vtbl,
            Invoke: unsafe extern "system" fn(
                this: *mut core::ffi::c_void,
                asyncinfo: *mut core::ffi::c_void,
                asyncstatus: AsyncStatus,
            ) -> windows_core::HRESULT,
            TResult: core::marker::PhantomData<TResult>,
        }
        #[repr(C)]
        struct AsyncOperationCompletedHandlerBox<
            TResult,
            F: FnMut(
                    windows_core::Ref<'_, IAsyncOperation<TResult>>,
                    AsyncStatus,
                ) -> windows_core::Result<()>
                + Send
                + 'static,
        >
        where
            TResult: windows_core::RuntimeType + 'static,
        {
            vtable: *const AsyncOperationCompletedHandler_Vtbl<TResult>,
            invoke: F,
            count: windows_core::imp::RefCount,
        }
        impl<
                TResult: windows_core::RuntimeType + 'static,
                F: FnMut(
                        windows_core::Ref<'_, IAsyncOperation<TResult>>,
                        AsyncStatus,
                    ) -> windows_core::Result<()>
                    + Send
                    + 'static,
            > AsyncOperationCompletedHandlerBox<TResult, F>
        {
            const VTABLE: AsyncOperationCompletedHandler_Vtbl<TResult> =
                AsyncOperationCompletedHandler_Vtbl::<TResult> {
                    base__: windows_core::IUnknown_Vtbl {
                        QueryInterface: Self::QueryInterface,
                        AddRef: Self::AddRef,
                        Release: Self::Release,
                    },
                    Invoke: Self::Invoke,
                    TResult: core::marker::PhantomData::<TResult>,
                };
            unsafe extern "system" fn QueryInterface(
                this: *mut core::ffi::c_void,
                iid: *const windows_core::GUID,
                interface: *mut *mut core::ffi::c_void,
            ) -> windows_core::HRESULT {
                unsafe {
                    let this = this as *mut *mut core::ffi::c_void as *mut Self;
                    if iid.is_null() || interface.is_null() {
                        return windows_core::HRESULT(-2147467261);
                    }
                    *interface = if *iid
                        == <AsyncOperationCompletedHandler<TResult> as windows_core::Interface>::IID
                        || *iid == <windows_core::IUnknown as windows_core::Interface>::IID
                        || *iid == <windows_core::imp::IAgileObject as windows_core::Interface>::IID
                    {
                        &mut (*this).vtable as *mut _ as _
                    } else if *iid == <windows_core::imp::IMarshal as windows_core::Interface>::IID
                    {
                        (*this).count.add_ref();
                        return windows_core::imp::marshaler(
                            core::mem::transmute(
                                &mut (*this).vtable as *mut _ as *mut core::ffi::c_void,
                            ),
                            interface,
                        );
                    } else {
                        core::ptr::null_mut()
                    };
                    if (*interface).is_null() {
                        windows_core::HRESULT(-2147467262)
                    } else {
                        (*this).count.add_ref();
                        windows_core::HRESULT(0)
                    }
                }
            }
            unsafe extern "system" fn AddRef(this: *mut core::ffi::c_void) -> u32 {
                unsafe {
                    let this = this as *mut *mut core::ffi::c_void as *mut Self;
                    (*this).count.add_ref()
                }
            }
            unsafe extern "system" fn Release(this: *mut core::ffi::c_void) -> u32 {
                unsafe {
                    let this = this as *mut *mut core::ffi::c_void as *mut Self;
                    let remaining = (*this).count.release();
                    if remaining == 0 {
                        let _ = windows_core::imp::Box::from_raw(this);
                    }
                    remaining
                }
            }
            unsafe extern "system" fn Invoke(
                this: *mut core::ffi::c_void,
                asyncinfo: *mut core::ffi::c_void,
                asyncstatus: AsyncStatus,
            ) -> windows_core::HRESULT {
                unsafe {
                    let this = &mut *(this as *mut *mut core::ffi::c_void as *mut Self);
                    (this.invoke)(core::mem::transmute_copy(&asyncinfo), asyncstatus).into()
                }
            }
        }
        #[repr(transparent)]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct AsyncOperationProgressHandler<TResult, TProgress>(
            windows_core::IUnknown,
            core::marker::PhantomData<TResult>,
            core::marker::PhantomData<TProgress>,
        )
        where
            TResult: windows_core::RuntimeType + 'static,
            TProgress: windows_core::RuntimeType + 'static;
        unsafe impl<
                TResult: windows_core::RuntimeType + 'static,
                TProgress: windows_core::RuntimeType + 'static,
            > windows_core::Interface for AsyncOperationProgressHandler<TResult, TProgress>
        {
            type Vtable = AsyncOperationProgressHandler_Vtbl<TResult, TProgress>;
            const IID: windows_core::GUID =
                windows_core::GUID::from_signature(<Self as windows_core::RuntimeType>::SIGNATURE);
        }
        impl<
                TResult: windows_core::RuntimeType + 'static,
                TProgress: windows_core::RuntimeType + 'static,
            > windows_core::RuntimeType for AsyncOperationProgressHandler<TResult, TProgress>
        {
            const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::new()
                .push_slice(b"pinterface({55690902-0aab-421a-8778-f8ce5026d758}")
                .push_slice(b";")
                .push_other(TResult::SIGNATURE)
                .push_slice(b";")
                .push_other(TProgress::SIGNATURE)
                .push_slice(b")");
        }
        impl<
                TResult: windows_core::RuntimeType + 'static,
                TProgress: windows_core::RuntimeType + 'static,
            > AsyncOperationProgressHandler<TResult, TProgress>
        {
            pub fn new<
                F: FnMut(
                        windows_core::Ref<'_, IAsyncOperationWithProgress<TResult, TProgress>>,
                        windows_core::Ref<'_, TProgress>,
                    ) -> windows_core::Result<()>
                    + Send
                    + 'static,
            >(
                invoke: F,
            ) -> Self {
                let com = AsyncOperationProgressHandlerBox {
                    vtable: &AsyncOperationProgressHandlerBox::<TResult, TProgress, F>::VTABLE,
                    count: windows_core::imp::RefCount::new(1),
                    invoke,
                };
                unsafe { core::mem::transmute(windows_core::imp::Box::new(com)) }
            }
            pub fn Invoke<P0, P1>(
                &self,
                asyncinfo: P0,
                progressinfo: P1,
            ) -> windows_core::Result<()>
            where
                P0: windows_core::Param<IAsyncOperationWithProgress<TResult, TProgress>>,
                P1: windows_core::Param<TProgress>,
            {
                let this = self;
                unsafe {
                    (windows_core::Interface::vtable(this).Invoke)(
                        windows_core::Interface::as_raw(this),
                        asyncinfo.param().abi(),
                        progressinfo.param().abi(),
                    )
                    .ok()
                }
            }
        }
        #[repr(C)]
        #[doc(hidden)]
        pub struct AsyncOperationProgressHandler_Vtbl<TResult, TProgress>
        where
            TResult: windows_core::RuntimeType + 'static,
            TProgress: windows_core::RuntimeType + 'static,
        {
            base__: windows_core::IUnknown_Vtbl,
            Invoke: unsafe extern "system" fn(
                this: *mut core::ffi::c_void,
                asyncinfo: *mut core::ffi::c_void,
                progressinfo: windows_core::AbiType<TProgress>,
            ) -> windows_core::HRESULT,
            TResult: core::marker::PhantomData<TResult>,
            TProgress: core::marker::PhantomData<TProgress>,
        }
        #[repr(C)]
        struct AsyncOperationProgressHandlerBox<
            TResult,
            TProgress,
            F: FnMut(
                    windows_core::Ref<'_, IAsyncOperationWithProgress<TResult, TProgress>>,
                    windows_core::Ref<'_, TProgress>,
                ) -> windows_core::Result<()>
                + Send
                + 'static,
        >
        where
            TResult: windows_core::RuntimeType + 'static,
            TProgress: windows_core::RuntimeType + 'static,
        {
            vtable: *const AsyncOperationProgressHandler_Vtbl<TResult, TProgress>,
            invoke: F,
            count: windows_core::imp::RefCount,
        }
        impl<
                TResult: windows_core::RuntimeType + 'static,
                TProgress: windows_core::RuntimeType + 'static,
                F: FnMut(
                        windows_core::Ref<'_, IAsyncOperationWithProgress<TResult, TProgress>>,
                        windows_core::Ref<'_, TProgress>,
                    ) -> windows_core::Result<()>
                    + Send
                    + 'static,
            > AsyncOperationProgressHandlerBox<TResult, TProgress, F>
        {
            const VTABLE: AsyncOperationProgressHandler_Vtbl<TResult, TProgress> =
                AsyncOperationProgressHandler_Vtbl::<TResult, TProgress> {
                    base__: windows_core::IUnknown_Vtbl {
                        QueryInterface: Self::QueryInterface,
                        AddRef: Self::AddRef,
                        Release: Self::Release,
                    },
                    Invoke: Self::Invoke,
                    TResult: core::marker::PhantomData::<TResult>,
                    TProgress: core::marker::PhantomData::<TProgress>,
                };
            unsafe extern "system" fn QueryInterface(
                this: *mut core::ffi::c_void,
                iid: *const windows_core::GUID,
                interface: *mut *mut core::ffi::c_void,
            ) -> windows_core::HRESULT {
                unsafe {
                    let this = this as *mut *mut core::ffi::c_void as *mut Self;
                    if iid.is_null() || interface.is_null() {
                        return windows_core::HRESULT(-2147467261);
                    }
                    * interface = if * iid == < AsyncOperationProgressHandler < TResult , TProgress > as windows_core::Interface >::IID || * iid == < windows_core::IUnknown as windows_core::Interface >::IID || * iid == < windows_core::imp::IAgileObject as windows_core::Interface >::IID { & mut ( * this ) . vtable as * mut _ as _ } else if * iid == < windows_core::imp::IMarshal as windows_core::Interface >::IID { ( * this ) . count . add_ref ( ) ; return windows_core::imp::marshaler ( core::mem::transmute ( & mut ( * this ) . vtable as * mut _ as * mut core::ffi::c_void ) , interface ) ; } else { core::ptr::null_mut ( ) } ;
                    if (*interface).is_null() {
                        windows_core::HRESULT(-2147467262)
                    } else {
                        (*this).count.add_ref();
                        windows_core::HRESULT(0)
                    }
                }
            }
            unsafe extern "system" fn AddRef(this: *mut core::ffi::c_void) -> u32 {
                unsafe {
                    let this = this as *mut *mut core::ffi::c_void as *mut Self;
                    (*this).count.add_ref()
                }
            }
            unsafe extern "system" fn Release(this: *mut core::ffi::c_void) -> u32 {
                unsafe {
                    let this = this as *mut *mut core::ffi::c_void as *mut Self;
                    let remaining = (*this).count.release();
                    if remaining == 0 {
                        let _ = windows_core::imp::Box::from_raw(this);
                    }
                    remaining
                }
            }
            unsafe extern "system" fn Invoke(
                this: *mut core::ffi::c_void,
                asyncinfo: *mut core::ffi::c_void,
                progressinfo: windows_core::AbiType<TProgress>,
            ) -> windows_core::HRESULT {
                unsafe {
                    let this = &mut *(this as *mut *mut core::ffi::c_void as *mut Self);
                    (this.invoke)(
                        core::mem::transmute_copy(&asyncinfo),
                        core::mem::transmute_copy(&progressinfo),
                    )
                    .into()
                }
            }
        }
        #[repr(transparent)]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct AsyncOperationWithProgressCompletedHandler<TResult, TProgress>(
            windows_core::IUnknown,
            core::marker::PhantomData<TResult>,
            core::marker::PhantomData<TProgress>,
        )
        where
            TResult: windows_core::RuntimeType + 'static,
            TProgress: windows_core::RuntimeType + 'static;
        unsafe impl<
                TResult: windows_core::RuntimeType + 'static,
                TProgress: windows_core::RuntimeType + 'static,
            > windows_core::Interface
            for AsyncOperationWithProgressCompletedHandler<TResult, TProgress>
        {
            type Vtable = AsyncOperationWithProgressCompletedHandler_Vtbl<TResult, TProgress>;
            const IID: windows_core::GUID =
                windows_core::GUID::from_signature(<Self as windows_core::RuntimeType>::SIGNATURE);
        }
        impl<
                TResult: windows_core::RuntimeType + 'static,
                TProgress: windows_core::RuntimeType + 'static,
            > windows_core::RuntimeType
            for AsyncOperationWithProgressCompletedHandler<TResult, TProgress>
        {
            const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::new()
                .push_slice(b"pinterface({e85df41d-6aa7-46e3-a8e2-f009d840c627}")
                .push_slice(b";")
                .push_other(TResult::SIGNATURE)
                .push_slice(b";")
                .push_other(TProgress::SIGNATURE)
                .push_slice(b")");
        }
        impl<
                TResult: windows_core::RuntimeType + 'static,
                TProgress: windows_core::RuntimeType + 'static,
            > AsyncOperationWithProgressCompletedHandler<TResult, TProgress>
        {
            pub fn new<
                F: FnMut(
                        windows_core::Ref<'_, IAsyncOperationWithProgress<TResult, TProgress>>,
                        AsyncStatus,
                    ) -> windows_core::Result<()>
                    + Send
                    + 'static,
            >(
                invoke: F,
            ) -> Self {
                let com =
                    AsyncOperationWithProgressCompletedHandlerBox {
                        vtable: &AsyncOperationWithProgressCompletedHandlerBox::<
                            TResult,
                            TProgress,
                            F,
                        >::VTABLE,
                        count: windows_core::imp::RefCount::new(1),
                        invoke,
                    };
                unsafe { core::mem::transmute(windows_core::imp::Box::new(com)) }
            }
            pub fn Invoke<P0>(
                &self,
                asyncinfo: P0,
                asyncstatus: AsyncStatus,
            ) -> windows_core::Result<()>
            where
                P0: windows_core::Param<IAsyncOperationWithProgress<TResult, TProgress>>,
            {
                let this = self;
                unsafe {
                    (windows_core::Interface::vtable(this).Invoke)(
                        windows_core::Interface::as_raw(this),
                        asyncinfo.param().abi(),
                        asyncstatus,
                    )
                    .ok()
                }
            }
        }
        #[repr(C)]
        #[doc(hidden)]
        pub struct AsyncOperationWithProgressCompletedHandler_Vtbl<TResult, TProgress>
        where
            TResult: windows_core::RuntimeType + 'static,
            TProgress: windows_core::RuntimeType + 'static,
        {
            base__: windows_core::IUnknown_Vtbl,
            Invoke: unsafe extern "system" fn(
                this: *mut core::ffi::c_void,
                asyncinfo: *mut core::ffi::c_void,
                asyncstatus: AsyncStatus,
            ) -> windows_core::HRESULT,
            TResult: core::marker::PhantomData<TResult>,
            TProgress: core::marker::PhantomData<TProgress>,
        }
        #[repr(C)]
        struct AsyncOperationWithProgressCompletedHandlerBox<
            TResult,
            TProgress,
            F: FnMut(
                    windows_core::Ref<'_, IAsyncOperationWithProgress<TResult, TProgress>>,
                    AsyncStatus,
                ) -> windows_core::Result<()>
                + Send
                + 'static,
        >
        where
            TResult: windows_core::RuntimeType + 'static,
            TProgress: windows_core::RuntimeType + 'static,
        {
            vtable: *const AsyncOperationWithProgressCompletedHandler_Vtbl<TResult, TProgress>,
            invoke: F,
            count: windows_core::imp::RefCount,
        }
        impl<
                TResult: windows_core::RuntimeType + 'static,
                TProgress: windows_core::RuntimeType + 'static,
                F: FnMut(
                        windows_core::Ref<'_, IAsyncOperationWithProgress<TResult, TProgress>>,
                        AsyncStatus,
                    ) -> windows_core::Result<()>
                    + Send
                    + 'static,
            > AsyncOperationWithProgressCompletedHandlerBox<TResult, TProgress, F>
        {
            const VTABLE: AsyncOperationWithProgressCompletedHandler_Vtbl<TResult, TProgress> =
                AsyncOperationWithProgressCompletedHandler_Vtbl::<TResult, TProgress> {
                    base__: windows_core::IUnknown_Vtbl {
                        QueryInterface: Self::QueryInterface,
                        AddRef: Self::AddRef,
                        Release: Self::Release,
                    },
                    Invoke: Self::Invoke,
                    TResult: core::marker::PhantomData::<TResult>,
                    TProgress: core::marker::PhantomData::<TProgress>,
                };
            unsafe extern "system" fn QueryInterface(
                this: *mut core::ffi::c_void,
                iid: *const windows_core::GUID,
                interface: *mut *mut core::ffi::c_void,
            ) -> windows_core::HRESULT {
                unsafe {
                    let this = this as *mut *mut core::ffi::c_void as *mut Self;
                    if iid.is_null() || interface.is_null() {
                        return windows_core::HRESULT(-2147467261);
                    }
                    *interface = if *iid == <AsyncOperationWithProgressCompletedHandler<
                        TResult,
                        TProgress,
                    > as windows_core::Interface>::IID
                        || *iid == <windows_core::IUnknown as windows_core::Interface>::IID
                        || *iid == <windows_core::imp::IAgileObject as windows_core::Interface>::IID
                    {
                        &mut (*this).vtable as *mut _ as _
                    } else if *iid == <windows_core::imp::IMarshal as windows_core::Interface>::IID
                    {
                        (*this).count.add_ref();
                        return windows_core::imp::marshaler(
                            core::mem::transmute(
                                &mut (*this).vtable as *mut _ as *mut core::ffi::c_void,
                            ),
                            interface,
                        );
                    } else {
                        core::ptr::null_mut()
                    };
                    if (*interface).is_null() {
                        windows_core::HRESULT(-2147467262)
                    } else {
                        (*this).count.add_ref();
                        windows_core::HRESULT(0)
                    }
                }
            }
            unsafe extern "system" fn AddRef(this: *mut core::ffi::c_void) -> u32 {
                unsafe {
                    let this = this as *mut *mut core::ffi::c_void as *mut Self;
                    (*this).count.add_ref()
                }
            }
            unsafe extern "system" fn Release(this: *mut core::ffi::c_void) -> u32 {
                unsafe {
                    let this = this as *mut *mut core::ffi::c_void as *mut Self;
                    let remaining = (*this).count.release();
                    if remaining == 0 {
                        let _ = windows_core::imp::Box::from_raw(this);
                    }
                    remaining
                }
            }
            unsafe extern "system" fn Invoke(
                this: *mut core::ffi::c_void,
                asyncinfo: *mut core::ffi::c_void,
                asyncstatus: AsyncStatus,
            ) -> windows_core::HRESULT {
                unsafe {
                    let this = &mut *(this as *mut *mut core::ffi::c_void as *mut Self);
                    (this.invoke)(core::mem::transmute_copy(&asyncinfo), asyncstatus).into()
                }
            }
        }
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        pub struct AsyncStatus(pub i32);
        impl AsyncStatus {
            pub const Canceled: Self = Self(2i32);
            pub const Completed: Self = Self(1i32);
            pub const Error: Self = Self(3i32);
            pub const Started: Self = Self(0i32);
        }
        impl windows_core::TypeKind for AsyncStatus {
            type TypeKind = windows_core::CopyType;
        }
        impl windows_core::RuntimeType for AsyncStatus {
            const SIGNATURE: windows_core::imp::ConstBuffer =
                windows_core::imp::ConstBuffer::from_slice(
                    b"enum(Windows.Foundation.AsyncStatus;i4)",
                );
        }
        windows_core::imp::define_interface!(
            IAsyncInfo,
            IAsyncInfo_Vtbl,
            0x00000036_0000_0000_c000_000000000046
        );
        impl windows_core::RuntimeType for IAsyncInfo {
            const SIGNATURE: windows_core::imp::ConstBuffer =
                windows_core::imp::ConstBuffer::for_interface::<Self>();
        }
        windows_core::imp::interface_hierarchy!(
            IAsyncInfo,
            windows_core::IUnknown,
            windows_core::IInspectable
        );
        impl IAsyncInfo {
            pub fn Id(&self) -> windows_core::Result<u32> {
                let this = self;
                unsafe {
                    let mut result__ = core::mem::zeroed();
                    (windows_core::Interface::vtable(this).Id)(
                        windows_core::Interface::as_raw(this),
                        &mut result__,
                    )
                    .map(|| result__)
                }
            }
            pub fn Status(&self) -> windows_core::Result<AsyncStatus> {
                let this = self;
                unsafe {
                    let mut result__ = core::mem::zeroed();
                    (windows_core::Interface::vtable(this).Status)(
                        windows_core::Interface::as_raw(this),
                        &mut result__,
                    )
                    .map(|| result__)
                }
            }
            pub fn ErrorCode(&self) -> windows_core::Result<windows_core::HRESULT> {
                let this = self;
                unsafe {
                    let mut result__ = core::mem::zeroed();
                    (windows_core::Interface::vtable(this).ErrorCode)(
                        windows_core::Interface::as_raw(this),
                        &mut result__,
                    )
                    .map(|| result__)
                }
            }
            pub fn Cancel(&self) -> windows_core::Result<()> {
                let this = self;
                unsafe {
                    (windows_core::Interface::vtable(this).Cancel)(windows_core::Interface::as_raw(
                        this,
                    ))
                    .ok()
                }
            }
            pub fn Close(&self) -> windows_core::Result<()> {
                let this = self;
                unsafe {
                    (windows_core::Interface::vtable(this).Close)(windows_core::Interface::as_raw(
                        this,
                    ))
                    .ok()
                }
            }
        }
        impl windows_core::RuntimeName for IAsyncInfo {
            const NAME: &'static str = "Windows.Foundation.IAsyncInfo";
        }
        pub trait IAsyncInfo_Impl: windows_core::IUnknownImpl {
            fn Id(&self) -> windows_core::Result<u32>;
            fn Status(&self) -> windows_core::Result<AsyncStatus>;
            fn ErrorCode(&self) -> windows_core::Result<windows_core::HRESULT>;
            fn Cancel(&self) -> windows_core::Result<()>;
            fn Close(&self) -> windows_core::Result<()>;
        }
        impl IAsyncInfo_Vtbl {
            pub const fn new<Identity: IAsyncInfo_Impl, const OFFSET: isize>() -> Self {
                unsafe extern "system" fn Id<Identity: IAsyncInfo_Impl, const OFFSET: isize>(
                    this: *mut core::ffi::c_void,
                    result__: *mut u32,
                ) -> windows_core::HRESULT {
                    unsafe {
                        let this: &Identity =
                            &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                        match IAsyncInfo_Impl::Id(this) {
                            Ok(ok__) => {
                                result__.write(core::mem::transmute_copy(&ok__));
                                windows_core::HRESULT(0)
                            }
                            Err(err) => err.into(),
                        }
                    }
                }
                unsafe extern "system" fn Status<Identity: IAsyncInfo_Impl, const OFFSET: isize>(
                    this: *mut core::ffi::c_void,
                    result__: *mut AsyncStatus,
                ) -> windows_core::HRESULT {
                    unsafe {
                        let this: &Identity =
                            &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                        match IAsyncInfo_Impl::Status(this) {
                            Ok(ok__) => {
                                result__.write(core::mem::transmute_copy(&ok__));
                                windows_core::HRESULT(0)
                            }
                            Err(err) => err.into(),
                        }
                    }
                }
                unsafe extern "system" fn ErrorCode<
                    Identity: IAsyncInfo_Impl,
                    const OFFSET: isize,
                >(
                    this: *mut core::ffi::c_void,
                    result__: *mut windows_core::HRESULT,
                ) -> windows_core::HRESULT {
                    unsafe {
                        let this: &Identity =
                            &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                        match IAsyncInfo_Impl::ErrorCode(this) {
                            Ok(ok__) => {
                                result__.write(core::mem::transmute_copy(&ok__));
                                windows_core::HRESULT(0)
                            }
                            Err(err) => err.into(),
                        }
                    }
                }
                unsafe extern "system" fn Cancel<Identity: IAsyncInfo_Impl, const OFFSET: isize>(
                    this: *mut core::ffi::c_void,
                ) -> windows_core::HRESULT {
                    unsafe {
                        let this: &Identity =
                            &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                        IAsyncInfo_Impl::Cancel(this).into()
                    }
                }
                unsafe extern "system" fn Close<Identity: IAsyncInfo_Impl, const OFFSET: isize>(
                    this: *mut core::ffi::c_void,
                ) -> windows_core::HRESULT {
                    unsafe {
                        let this: &Identity =
                            &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                        IAsyncInfo_Impl::Close(this).into()
                    }
                }
                Self {
                    base__: windows_core::IInspectable_Vtbl::new::<Identity, IAsyncInfo, OFFSET>(),
                    Id: Id::<Identity, OFFSET>,
                    Status: Status::<Identity, OFFSET>,
                    ErrorCode: ErrorCode::<Identity, OFFSET>,
                    Cancel: Cancel::<Identity, OFFSET>,
                    Close: Close::<Identity, OFFSET>,
                }
            }
            pub fn matches(iid: &windows_core::GUID) -> bool {
                iid == &<IAsyncInfo as windows_core::Interface>::IID
            }
        }
        #[repr(C)]
        #[doc(hidden)]
        pub struct IAsyncInfo_Vtbl {
            pub base__: windows_core::IInspectable_Vtbl,
            pub Id: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                *mut u32,
            ) -> windows_core::HRESULT,
            pub Status: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                *mut AsyncStatus,
            ) -> windows_core::HRESULT,
            pub ErrorCode: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                *mut windows_core::HRESULT,
            ) -> windows_core::HRESULT,
            pub Cancel: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
            pub Close: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
        }
        #[repr(transparent)]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct IAsyncOperation<TResult>(
            windows_core::IUnknown,
            core::marker::PhantomData<TResult>,
        )
        where
            TResult: windows_core::RuntimeType + 'static;
        impl<TResult: windows_core::RuntimeType + 'static>
            windows_core::imp::CanInto<windows_core::IUnknown> for IAsyncOperation<TResult>
        {
        }
        impl<TResult: windows_core::RuntimeType + 'static>
            windows_core::imp::CanInto<windows_core::IInspectable> for IAsyncOperation<TResult>
        {
        }
        unsafe impl<TResult: windows_core::RuntimeType + 'static> windows_core::Interface
            for IAsyncOperation<TResult>
        {
            type Vtable = IAsyncOperation_Vtbl<TResult>;
            const IID: windows_core::GUID =
                windows_core::GUID::from_signature(<Self as windows_core::RuntimeType>::SIGNATURE);
        }
        impl<TResult: windows_core::RuntimeType + 'static> windows_core::RuntimeType
            for IAsyncOperation<TResult>
        {
            const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::new()
                .push_slice(b"pinterface({9fc2b0bb-e446-44e2-aa61-9cab8f636af2}")
                .push_slice(b";")
                .push_other(TResult::SIGNATURE)
                .push_slice(b")");
        }
        impl<TResult: windows_core::RuntimeType + 'static> windows_core::imp::CanInto<IAsyncInfo>
            for IAsyncOperation<TResult>
        {
            const QUERY: bool = true;
        }
        impl<TResult: windows_core::RuntimeType + 'static> IAsyncOperation<TResult> {
            pub fn SetCompleted<P0>(&self, handler: P0) -> windows_core::Result<()>
            where
                P0: windows_core::Param<AsyncOperationCompletedHandler<TResult>>,
            {
                let this = self;
                unsafe {
                    (windows_core::Interface::vtable(this).SetCompleted)(
                        windows_core::Interface::as_raw(this),
                        handler.param().abi(),
                    )
                    .ok()
                }
            }
            pub fn Completed(
                &self,
            ) -> windows_core::Result<AsyncOperationCompletedHandler<TResult>> {
                let this = self;
                unsafe {
                    let mut result__ = core::mem::zeroed();
                    (windows_core::Interface::vtable(this).Completed)(
                        windows_core::Interface::as_raw(this),
                        &mut result__,
                    )
                    .and_then(|| windows_core::Type::from_abi(result__))
                }
            }
            pub fn GetResults(&self) -> windows_core::Result<TResult> {
                let this = self;
                unsafe {
                    let mut result__ = core::mem::zeroed();
                    (windows_core::Interface::vtable(this).GetResults)(
                        windows_core::Interface::as_raw(this),
                        &mut result__,
                    )
                    .and_then(|| windows_core::Type::from_abi(result__))
                }
            }
            pub fn Id(&self) -> windows_core::Result<u32> {
                let this = &windows_core::Interface::cast::<IAsyncInfo>(self)?;
                unsafe {
                    let mut result__ = core::mem::zeroed();
                    (windows_core::Interface::vtable(this).Id)(
                        windows_core::Interface::as_raw(this),
                        &mut result__,
                    )
                    .map(|| result__)
                }
            }
            pub fn Status(&self) -> windows_core::Result<AsyncStatus> {
                let this = &windows_core::Interface::cast::<IAsyncInfo>(self)?;
                unsafe {
                    let mut result__ = core::mem::zeroed();
                    (windows_core::Interface::vtable(this).Status)(
                        windows_core::Interface::as_raw(this),
                        &mut result__,
                    )
                    .map(|| result__)
                }
            }
            pub fn ErrorCode(&self) -> windows_core::Result<windows_core::HRESULT> {
                let this = &windows_core::Interface::cast::<IAsyncInfo>(self)?;
                unsafe {
                    let mut result__ = core::mem::zeroed();
                    (windows_core::Interface::vtable(this).ErrorCode)(
                        windows_core::Interface::as_raw(this),
                        &mut result__,
                    )
                    .map(|| result__)
                }
            }
            pub fn Cancel(&self) -> windows_core::Result<()> {
                let this = &windows_core::Interface::cast::<IAsyncInfo>(self)?;
                unsafe {
                    (windows_core::Interface::vtable(this).Cancel)(windows_core::Interface::as_raw(
                        this,
                    ))
                    .ok()
                }
            }
            pub fn Close(&self) -> windows_core::Result<()> {
                let this = &windows_core::Interface::cast::<IAsyncInfo>(self)?;
                unsafe {
                    (windows_core::Interface::vtable(this).Close)(windows_core::Interface::as_raw(
                        this,
                    ))
                    .ok()
                }
            }
        }
        unsafe impl<TResult: windows_core::RuntimeType + 'static> Send for IAsyncOperation<TResult> {}
        unsafe impl<TResult: windows_core::RuntimeType + 'static> Sync for IAsyncOperation<TResult> {}
        impl<TResult: windows_core::RuntimeType + 'static> windows_core::RuntimeName
            for IAsyncOperation<TResult>
        {
            const NAME: &'static str = "Windows.Foundation.IAsyncOperation";
        }
        pub trait IAsyncOperation_Impl<TResult>: IAsyncInfo_Impl
        where
            TResult: windows_core::RuntimeType + 'static,
        {
            fn SetCompleted(
                &self,
                handler: windows_core::Ref<'_, AsyncOperationCompletedHandler<TResult>>,
            ) -> windows_core::Result<()>;
            fn Completed(&self) -> windows_core::Result<AsyncOperationCompletedHandler<TResult>>;
            fn GetResults(&self) -> windows_core::Result<TResult>;
        }
        impl<TResult: windows_core::RuntimeType + 'static> IAsyncOperation_Vtbl<TResult> {
            pub const fn new<Identity: IAsyncOperation_Impl<TResult>, const OFFSET: isize>() -> Self
            {
                unsafe extern "system" fn SetCompleted<
                    TResult: windows_core::RuntimeType + 'static,
                    Identity: IAsyncOperation_Impl<TResult>,
                    const OFFSET: isize,
                >(
                    this: *mut core::ffi::c_void,
                    handler: *mut core::ffi::c_void,
                ) -> windows_core::HRESULT {
                    unsafe {
                        let this: &Identity =
                            &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                        IAsyncOperation_Impl::SetCompleted(
                            this,
                            core::mem::transmute_copy(&handler),
                        )
                        .into()
                    }
                }
                unsafe extern "system" fn Completed<
                    TResult: windows_core::RuntimeType + 'static,
                    Identity: IAsyncOperation_Impl<TResult>,
                    const OFFSET: isize,
                >(
                    this: *mut core::ffi::c_void,
                    result__: *mut *mut core::ffi::c_void,
                ) -> windows_core::HRESULT {
                    unsafe {
                        let this: &Identity =
                            &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                        match IAsyncOperation_Impl::Completed(this) {
                            Ok(ok__) => {
                                result__.write(core::mem::transmute_copy(&ok__));
                                core::mem::forget(ok__);
                                windows_core::HRESULT(0)
                            }
                            Err(err) => err.into(),
                        }
                    }
                }
                unsafe extern "system" fn GetResults<
                    TResult: windows_core::RuntimeType + 'static,
                    Identity: IAsyncOperation_Impl<TResult>,
                    const OFFSET: isize,
                >(
                    this: *mut core::ffi::c_void,
                    result__: *mut windows_core::AbiType<TResult>,
                ) -> windows_core::HRESULT {
                    unsafe {
                        let this: &Identity =
                            &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                        match IAsyncOperation_Impl::GetResults(this) {
                            Ok(ok__) => {
                                result__.write(core::mem::transmute_copy(&ok__));
                                core::mem::forget(ok__);
                                windows_core::HRESULT(0)
                            }
                            Err(err) => err.into(),
                        }
                    }
                }
                Self {
                    base__: windows_core::IInspectable_Vtbl::new::<
                        Identity,
                        IAsyncOperation<TResult>,
                        OFFSET,
                    >(),
                    SetCompleted: SetCompleted::<TResult, Identity, OFFSET>,
                    Completed: Completed::<TResult, Identity, OFFSET>,
                    GetResults: GetResults::<TResult, Identity, OFFSET>,
                    TResult: core::marker::PhantomData::<TResult>,
                }
            }
            pub fn matches(iid: &windows_core::GUID) -> bool {
                iid == &<IAsyncOperation<TResult> as windows_core::Interface>::IID
            }
        }
        #[repr(C)]
        #[doc(hidden)]
        pub struct IAsyncOperation_Vtbl<TResult>
        where
            TResult: windows_core::RuntimeType + 'static,
        {
            pub base__: windows_core::IInspectable_Vtbl,
            pub SetCompleted: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                *mut core::ffi::c_void,
            ) -> windows_core::HRESULT,
            pub Completed: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                *mut *mut core::ffi::c_void,
            ) -> windows_core::HRESULT,
            pub GetResults: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                *mut windows_core::AbiType<TResult>,
            ) -> windows_core::HRESULT,
            TResult: core::marker::PhantomData<TResult>,
        }
        #[repr(transparent)]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct IAsyncOperationWithProgress<TResult, TProgress>(
            windows_core::IUnknown,
            core::marker::PhantomData<TResult>,
            core::marker::PhantomData<TProgress>,
        )
        where
            TResult: windows_core::RuntimeType + 'static,
            TProgress: windows_core::RuntimeType + 'static;
        impl<
                TResult: windows_core::RuntimeType + 'static,
                TProgress: windows_core::RuntimeType + 'static,
            > windows_core::imp::CanInto<windows_core::IUnknown>
            for IAsyncOperationWithProgress<TResult, TProgress>
        {
        }
        impl<
                TResult: windows_core::RuntimeType + 'static,
                TProgress: windows_core::RuntimeType + 'static,
            > windows_core::imp::CanInto<windows_core::IInspectable>
            for IAsyncOperationWithProgress<TResult, TProgress>
        {
        }
        unsafe impl<
                TResult: windows_core::RuntimeType + 'static,
                TProgress: windows_core::RuntimeType + 'static,
            > windows_core::Interface for IAsyncOperationWithProgress<TResult, TProgress>
        {
            type Vtable = IAsyncOperationWithProgress_Vtbl<TResult, TProgress>;
            const IID: windows_core::GUID =
                windows_core::GUID::from_signature(<Self as windows_core::RuntimeType>::SIGNATURE);
        }
        impl<
                TResult: windows_core::RuntimeType + 'static,
                TProgress: windows_core::RuntimeType + 'static,
            > windows_core::RuntimeType for IAsyncOperationWithProgress<TResult, TProgress>
        {
            const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::new()
                .push_slice(b"pinterface({b5d036d7-e297-498f-ba60-0289e76e23dd}")
                .push_slice(b";")
                .push_other(TResult::SIGNATURE)
                .push_slice(b";")
                .push_other(TProgress::SIGNATURE)
                .push_slice(b")");
        }
        impl<
                TResult: windows_core::RuntimeType + 'static,
                TProgress: windows_core::RuntimeType + 'static,
            > windows_core::imp::CanInto<IAsyncInfo>
            for IAsyncOperationWithProgress<TResult, TProgress>
        {
            const QUERY: bool = true;
        }
        impl<
                TResult: windows_core::RuntimeType + 'static,
                TProgress: windows_core::RuntimeType + 'static,
            > IAsyncOperationWithProgress<TResult, TProgress>
        {
            pub fn SetProgress<P0>(&self, handler: P0) -> windows_core::Result<()>
            where
                P0: windows_core::Param<AsyncOperationProgressHandler<TResult, TProgress>>,
            {
                let this = self;
                unsafe {
                    (windows_core::Interface::vtable(this).SetProgress)(
                        windows_core::Interface::as_raw(this),
                        handler.param().abi(),
                    )
                    .ok()
                }
            }
            pub fn Progress(
                &self,
            ) -> windows_core::Result<AsyncOperationProgressHandler<TResult, TProgress>>
            {
                let this = self;
                unsafe {
                    let mut result__ = core::mem::zeroed();
                    (windows_core::Interface::vtable(this).Progress)(
                        windows_core::Interface::as_raw(this),
                        &mut result__,
                    )
                    .and_then(|| windows_core::Type::from_abi(result__))
                }
            }
            pub fn SetCompleted<P0>(&self, handler: P0) -> windows_core::Result<()>
            where
                P0: windows_core::Param<
                    AsyncOperationWithProgressCompletedHandler<TResult, TProgress>,
                >,
            {
                let this = self;
                unsafe {
                    (windows_core::Interface::vtable(this).SetCompleted)(
                        windows_core::Interface::as_raw(this),
                        handler.param().abi(),
                    )
                    .ok()
                }
            }
            pub fn Completed(
                &self,
            ) -> windows_core::Result<AsyncOperationWithProgressCompletedHandler<TResult, TProgress>>
            {
                let this = self;
                unsafe {
                    let mut result__ = core::mem::zeroed();
                    (windows_core::Interface::vtable(this).Completed)(
                        windows_core::Interface::as_raw(this),
                        &mut result__,
                    )
                    .and_then(|| windows_core::Type::from_abi(result__))
                }
            }
            pub fn GetResults(&self) -> windows_core::Result<TResult> {
                let this = self;
                unsafe {
                    let mut result__ = core::mem::zeroed();
                    (windows_core::Interface::vtable(this).GetResults)(
                        windows_core::Interface::as_raw(this),
                        &mut result__,
                    )
                    .and_then(|| windows_core::Type::from_abi(result__))
                }
            }
            pub fn Id(&self) -> windows_core::Result<u32> {
                let this = &windows_core::Interface::cast::<IAsyncInfo>(self)?;
                unsafe {
                    let mut result__ = core::mem::zeroed();
                    (windows_core::Interface::vtable(this).Id)(
                        windows_core::Interface::as_raw(this),
                        &mut result__,
                    )
                    .map(|| result__)
                }
            }
            pub fn Status(&self) -> windows_core::Result<AsyncStatus> {
                let this = &windows_core::Interface::cast::<IAsyncInfo>(self)?;
                unsafe {
                    let mut result__ = core::mem::zeroed();
                    (windows_core::Interface::vtable(this).Status)(
                        windows_core::Interface::as_raw(this),
                        &mut result__,
                    )
                    .map(|| result__)
                }
            }
            pub fn ErrorCode(&self) -> windows_core::Result<windows_core::HRESULT> {
                let this = &windows_core::Interface::cast::<IAsyncInfo>(self)?;
                unsafe {
                    let mut result__ = core::mem::zeroed();
                    (windows_core::Interface::vtable(this).ErrorCode)(
                        windows_core::Interface::as_raw(this),
                        &mut result__,
                    )
                    .map(|| result__)
                }
            }
            pub fn Cancel(&self) -> windows_core::Result<()> {
                let this = &windows_core::Interface::cast::<IAsyncInfo>(self)?;
                unsafe {
                    (windows_core::Interface::vtable(this).Cancel)(windows_core::Interface::as_raw(
                        this,
                    ))
                    .ok()
                }
            }
            pub fn Close(&self) -> windows_core::Result<()> {
                let this = &windows_core::Interface::cast::<IAsyncInfo>(self)?;
                unsafe {
                    (windows_core::Interface::vtable(this).Close)(windows_core::Interface::as_raw(
                        this,
                    ))
                    .ok()
                }
            }
        }
        unsafe impl<
                TResult: windows_core::RuntimeType + 'static,
                TProgress: windows_core::RuntimeType + 'static,
            > Send for IAsyncOperationWithProgress<TResult, TProgress>
        {
        }
        unsafe impl<
                TResult: windows_core::RuntimeType + 'static,
                TProgress: windows_core::RuntimeType + 'static,
            > Sync for IAsyncOperationWithProgress<TResult, TProgress>
        {
        }
        impl<
                TResult: windows_core::RuntimeType + 'static,
                TProgress: windows_core::RuntimeType + 'static,
            > windows_core::RuntimeName for IAsyncOperationWithProgress<TResult, TProgress>
        {
            const NAME: &'static str = "Windows.Foundation.IAsyncOperationWithProgress";
        }
        pub trait IAsyncOperationWithProgress_Impl<TResult, TProgress>: IAsyncInfo_Impl
        where
            TResult: windows_core::RuntimeType + 'static,
            TProgress: windows_core::RuntimeType + 'static,
        {
            fn SetProgress(
                &self,
                handler: windows_core::Ref<'_, AsyncOperationProgressHandler<TResult, TProgress>>,
            ) -> windows_core::Result<()>;
            fn Progress(
                &self,
            ) -> windows_core::Result<AsyncOperationProgressHandler<TResult, TProgress>>;
            fn SetCompleted(
                &self,
                handler: windows_core::Ref<
                    '_,
                    AsyncOperationWithProgressCompletedHandler<TResult, TProgress>,
                >,
            ) -> windows_core::Result<()>;
            fn Completed(
                &self,
            ) -> windows_core::Result<AsyncOperationWithProgressCompletedHandler<TResult, TProgress>>;
            fn GetResults(&self) -> windows_core::Result<TResult>;
        }
        impl<
                TResult: windows_core::RuntimeType + 'static,
                TProgress: windows_core::RuntimeType + 'static,
            > IAsyncOperationWithProgress_Vtbl<TResult, TProgress>
        {
            pub const fn new<
                Identity: IAsyncOperationWithProgress_Impl<TResult, TProgress>,
                const OFFSET: isize,
            >() -> Self {
                unsafe extern "system" fn SetProgress<
                    TResult: windows_core::RuntimeType + 'static,
                    TProgress: windows_core::RuntimeType + 'static,
                    Identity: IAsyncOperationWithProgress_Impl<TResult, TProgress>,
                    const OFFSET: isize,
                >(
                    this: *mut core::ffi::c_void,
                    handler: *mut core::ffi::c_void,
                ) -> windows_core::HRESULT {
                    unsafe {
                        let this: &Identity =
                            &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                        IAsyncOperationWithProgress_Impl::SetProgress(
                            this,
                            core::mem::transmute_copy(&handler),
                        )
                        .into()
                    }
                }
                unsafe extern "system" fn Progress<
                    TResult: windows_core::RuntimeType + 'static,
                    TProgress: windows_core::RuntimeType + 'static,
                    Identity: IAsyncOperationWithProgress_Impl<TResult, TProgress>,
                    const OFFSET: isize,
                >(
                    this: *mut core::ffi::c_void,
                    result__: *mut *mut core::ffi::c_void,
                ) -> windows_core::HRESULT {
                    unsafe {
                        let this: &Identity =
                            &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                        match IAsyncOperationWithProgress_Impl::Progress(this) {
                            Ok(ok__) => {
                                result__.write(core::mem::transmute_copy(&ok__));
                                core::mem::forget(ok__);
                                windows_core::HRESULT(0)
                            }
                            Err(err) => err.into(),
                        }
                    }
                }
                unsafe extern "system" fn SetCompleted<
                    TResult: windows_core::RuntimeType + 'static,
                    TProgress: windows_core::RuntimeType + 'static,
                    Identity: IAsyncOperationWithProgress_Impl<TResult, TProgress>,
                    const OFFSET: isize,
                >(
                    this: *mut core::ffi::c_void,
                    handler: *mut core::ffi::c_void,
                ) -> windows_core::HRESULT {
                    unsafe {
                        let this: &Identity =
                            &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                        IAsyncOperationWithProgress_Impl::SetCompleted(
                            this,
                            core::mem::transmute_copy(&handler),
                        )
                        .into()
                    }
                }
                unsafe extern "system" fn Completed<
                    TResult: windows_core::RuntimeType + 'static,
                    TProgress: windows_core::RuntimeType + 'static,
                    Identity: IAsyncOperationWithProgress_Impl<TResult, TProgress>,
                    const OFFSET: isize,
                >(
                    this: *mut core::ffi::c_void,
                    result__: *mut *mut core::ffi::c_void,
                ) -> windows_core::HRESULT {
                    unsafe {
                        let this: &Identity =
                            &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                        match IAsyncOperationWithProgress_Impl::Completed(this) {
                            Ok(ok__) => {
                                result__.write(core::mem::transmute_copy(&ok__));
                                core::mem::forget(ok__);
                                windows_core::HRESULT(0)
                            }
                            Err(err) => err.into(),
                        }
                    }
                }
                unsafe extern "system" fn GetResults<
                    TResult: windows_core::RuntimeType + 'static,
                    TProgress: windows_core::RuntimeType + 'static,
                    Identity: IAsyncOperationWithProgress_Impl<TResult, TProgress>,
                    const OFFSET: isize,
                >(
                    this: *mut core::ffi::c_void,
                    result__: *mut windows_core::AbiType<TResult>,
                ) -> windows_core::HRESULT {
                    unsafe {
                        let this: &Identity =
                            &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                        match IAsyncOperationWithProgress_Impl::GetResults(this) {
                            Ok(ok__) => {
                                result__.write(core::mem::transmute_copy(&ok__));
                                core::mem::forget(ok__);
                                windows_core::HRESULT(0)
                            }
                            Err(err) => err.into(),
                        }
                    }
                }
                Self {
                    base__: windows_core::IInspectable_Vtbl::new::<
                        Identity,
                        IAsyncOperationWithProgress<TResult, TProgress>,
                        OFFSET,
                    >(),
                    SetProgress: SetProgress::<TResult, TProgress, Identity, OFFSET>,
                    Progress: Progress::<TResult, TProgress, Identity, OFFSET>,
                    SetCompleted: SetCompleted::<TResult, TProgress, Identity, OFFSET>,
                    Completed: Completed::<TResult, TProgress, Identity, OFFSET>,
                    GetResults: GetResults::<TResult, TProgress, Identity, OFFSET>,
                    TResult: core::marker::PhantomData::<TResult>,
                    TProgress: core::marker::PhantomData::<TProgress>,
                }
            }
            pub fn matches(iid: &windows_core::GUID) -> bool {
                iid == & < IAsyncOperationWithProgress < TResult , TProgress > as windows_core::Interface >::IID
            }
        }
        #[repr(C)]
        #[doc(hidden)]
        pub struct IAsyncOperationWithProgress_Vtbl<TResult, TProgress>
        where
            TResult: windows_core::RuntimeType + 'static,
            TProgress: windows_core::RuntimeType + 'static,
        {
            pub base__: windows_core::IInspectable_Vtbl,
            pub SetProgress: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                *mut core::ffi::c_void,
            ) -> windows_core::HRESULT,
            pub Progress: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                *mut *mut core::ffi::c_void,
            ) -> windows_core::HRESULT,
            pub SetCompleted: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                *mut core::ffi::c_void,
            ) -> windows_core::HRESULT,
            pub Completed: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                *mut *mut core::ffi::c_void,
            ) -> windows_core::HRESULT,
            pub GetResults: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                *mut windows_core::AbiType<TResult>,
            ) -> windows_core::HRESULT,
            TResult: core::marker::PhantomData<TResult>,
            TProgress: core::marker::PhantomData<TProgress>,
        }
        windows_core::imp::define_interface!(
            IClosable,
            IClosable_Vtbl,
            0x30d5a829_7fa4_4026_83bb_d75bae4ea99e
        );
        impl windows_core::RuntimeType for IClosable {
            const SIGNATURE: windows_core::imp::ConstBuffer =
                windows_core::imp::ConstBuffer::for_interface::<Self>();
        }
        windows_core::imp::interface_hierarchy!(
            IClosable,
            windows_core::IUnknown,
            windows_core::IInspectable
        );
        impl IClosable {
            pub fn Close(&self) -> windows_core::Result<()> {
                let this = self;
                unsafe {
                    (windows_core::Interface::vtable(this).Close)(windows_core::Interface::as_raw(
                        this,
                    ))
                    .ok()
                }
            }
        }
        impl windows_core::RuntimeName for IClosable {
            const NAME: &'static str = "Windows.Foundation.IClosable";
        }
        pub trait IClosable_Impl: windows_core::IUnknownImpl {
            fn Close(&self) -> windows_core::Result<()>;
        }
        impl IClosable_Vtbl {
            pub const fn new<Identity: IClosable_Impl, const OFFSET: isize>() -> Self {
                unsafe extern "system" fn Close<Identity: IClosable_Impl, const OFFSET: isize>(
                    this: *mut core::ffi::c_void,
                ) -> windows_core::HRESULT {
                    unsafe {
                        let this: &Identity =
                            &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                        IClosable_Impl::Close(this).into()
                    }
                }
                Self {
                    base__: windows_core::IInspectable_Vtbl::new::<Identity, IClosable, OFFSET>(),
                    Close: Close::<Identity, OFFSET>,
                }
            }
            pub fn matches(iid: &windows_core::GUID) -> bool {
                iid == &<IClosable as windows_core::Interface>::IID
            }
        }
        #[repr(C)]
        #[doc(hidden)]
        pub struct IClosable_Vtbl {
            pub base__: windows_core::IInspectable_Vtbl,
            pub Close: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
        }
    }
}
