use sailfish::TemplateSimple;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TemplateSimple)]
#[template(path = "hello.stpl")]
pub struct TemplateWorkaround {
    name: String,
    first_name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestTemplate {
    name: String,
    first_name: String,
}

impl TestTemplate {
    pub fn new(name: String, first_name: String) -> Self {
        Self { name, first_name }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl sailfish::TemplateOnce for TestTemplate {
    fn render_once(self) -> sailfish::RenderResult {
        use sailfish::runtime::{Buffer, SizeHint};
        static SIZE_HINT: SizeHint = SizeHint::new();

        let mut buf = Buffer::with_capacity(SIZE_HINT.get());
        self.render_once_to(&mut buf)?;
        SIZE_HINT.update(buf.len());

        Ok(buf.into_string())
    }

    fn render_once_to(
        self,
        buf: &mut sailfish::runtime::Buffer,
    ) -> Result<(), sailfish::runtime::RenderError> {
        use sailfish::runtime::RenderError;
        use shared::Postcard;
        let mut lock = shared::WASM_INSTANCE.lock().unwrap();
        let module = std::module_path!();
        let module = module.split_once("::").map_or(module, |s| s.0);
        let instance = lock.get_mut(module);

        if let Some(instance) = instance {
            match instance
                .call::<Postcard<TestTemplate>, String>("render_test_template", Postcard(self))
            {
                Ok(result) => {
                    buf.push_str(&result);
                    Ok(())
                }
                Err(err) => Err(RenderError::Msg(err.to_string())),
            }
        } else {
            Err(RenderError::new("instance not found"))
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl sailfish::TemplateOnce for TestTemplate {
    fn render_once(self) -> sailfish::RenderResult {
        TemplateWorkaround {
            name: self.name,
            first_name: self.first_name,
        }
        .render_once()
    }

    fn render_once_to(
        self,
        buf: &mut sailfish::runtime::Buffer,
    ) -> Result<(), sailfish::runtime::RenderError> {
        TemplateWorkaround {
            name: self.name,
            first_name: self.first_name,
        }
        .render_once_to(buf)
    }
}

#[cfg(target_arch = "wasm32")]
#[extism_pdk::plugin_fn]
pub fn render_test_template(
    template: shared::Postcard<TestTemplate>,
) -> extism_pdk::FnResult<String> {
    let template = template.into_inner();

    sailfish::TemplateOnce::render_once(template).map_err(Into::into)
}
