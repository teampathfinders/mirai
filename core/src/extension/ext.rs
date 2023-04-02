use wasmtime::{Instance, Module, Store, TypedFunc};

struct ExtensionFnPointers {
    alloc_fn: TypedFunc<u32, i32>,
    dealloc_fn: TypedFunc<(i32, u32), ()>,
    realloc_fn: TypedFunc<(i32, u32, u32), i32>
}

pub struct Extension {
    pub name: String,
    pub version: [u8; 4],
    fn_pointers: ExtensionFnPointers
}

impl Extension {
    pub fn new(instance: &Instance, mut store: &mut Store<()>) -> wasmtime::Result<Self> {
        let alloc_fn = instance.get_typed_func::<u32, i32>(&mut store, "__pyro_alloc")?;
        let dealloc_fn = instance.get_typed_func::<(i32, u32), ()>(&mut store, "__pyro_dealloc")?;
        let realloc_fn = instance.get_typed_func::<(i32, u32, u32), i32>(&mut store, "__pyro_realloc")?;

        let name = {
            let name_len_fn = instance.get_typed_func::<(), u32>(&mut store, "__pyro_ext_name_len")?;
            let len = name_len_fn.call(&mut store, ())?;

            let name_fn = instance.get_typed_func::<i32, ()>(&mut store, "__pyro_ext_name")?;
            let name_ptr = alloc_fn.call(&mut store, len)?;
            name_fn.call(&mut store, name_ptr)?;

            let linear_mem = instance.get_memory(&mut store, "memory").ok_or_else(||
                wasmtime::Error::msg("Unable to retrieve extension name")
                    .context("linear memory block 'memory' was not found")
            )?;

            let mut utf8_buffer = vec![0; len as usize];
            linear_mem.read(&mut store, name_ptr as usize, &mut utf8_buffer)?;

            // FIXME: Add proper error handling
            let name = String::from_utf8(utf8_buffer).unwrap();

            dealloc_fn.call(&mut store, (name_ptr, len))?;

            name
        };

        let version = {
            let version_fn = instance.get_typed_func::<(), u32>(&mut store, "__pyro_ext_version")?;
            version_fn.call(store, ())?.to_le_bytes()
        };

        Ok(Self {
            name, version,
            fn_pointers: ExtensionFnPointers {
                alloc_fn, dealloc_fn, realloc_fn
            }
        })
    }
}