#[trilogy_derive::module(crate_name=crate)]
pub mod fs {
    use crate::{Result, Runtime};
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use trilogy_vm::{Array, Value};

    #[derive(Clone)]
    pub struct File {
        path: PathBuf,
        file: Arc<Mutex<Option<std::fs::File>>>,
    }

    #[trilogy_derive::func(crate_name=crate)]
    pub fn file(rt: Runtime, path: Value) -> Result<()> {
        let path = rt.typecheck::<String>(path)?;
        let path = path
            .parse::<PathBuf>()
            .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
        rt.r#return(File {
            path,
            file: Arc::new(Mutex::new(None)),
        })
    }

    #[trilogy_derive::proc(crate_name=crate)]
    pub fn read(rt: Runtime, path: Value) -> Result<()> {
        let path = rt.typecheck::<String>(path)?;
        let path = path
            .parse::<PathBuf>()
            .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
        let string = std::fs::read_to_string(path)
            .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
        rt.r#return(string)
    }

    #[trilogy_derive::proc(crate_name=crate)]
    pub fn write(rt: Runtime, path: Value, contents: Value) -> Result<()> {
        let path = rt.typecheck::<String>(path)?;
        let contents = rt.typecheck::<String>(contents)?;
        let path = path
            .parse::<PathBuf>()
            .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
        std::fs::write(path, contents)
            .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
        rt.r#return(())
    }

    #[trilogy_derive::proc(crate_name=crate)]
    pub fn copy(rt: Runtime, src: Value, dest: Value) -> Result<()> {
        let src = rt.typecheck::<String>(src)?;
        let dest = rt.typecheck::<String>(dest)?;
        let src = src
            .parse::<PathBuf>()
            .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
        let dest = dest
            .parse::<PathBuf>()
            .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
        std::fs::copy(src, dest)
            .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
        rt.r#return(())
    }

    #[trilogy_derive::proc(crate_name=crate, name="move")]
    pub fn r#move(rt: Runtime, src: Value, dest: Value) -> Result<()> {
        let src = rt.typecheck::<String>(src)?;
        let dest = rt.typecheck::<String>(dest)?;
        let src = src
            .parse::<PathBuf>()
            .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
        let dest = dest
            .parse::<PathBuf>()
            .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
        std::fs::rename(src, dest)
            .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
        rt.r#return(())
    }

    #[trilogy_derive::proc(crate_name=crate)]
    pub fn create_dir(rt: Runtime, path: Value) -> Result<()> {
        let path = rt.typecheck::<String>(path)?;
        let path = path
            .parse::<PathBuf>()
            .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
        std::fs::create_dir(path)
            .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
        rt.r#return(())
    }

    #[trilogy_derive::proc(crate_name=crate)]
    pub fn create_dir_all(rt: Runtime, path: Value) -> Result<()> {
        let path = rt.typecheck::<String>(path)?;
        let path = path
            .parse::<PathBuf>()
            .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
        std::fs::create_dir_all(path)
            .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
        rt.r#return(())
    }

    #[trilogy_derive::proc(crate_name=crate)]
    pub fn remove(rt: Runtime, path: Value) -> Result<()> {
        let path = rt.typecheck::<String>(path)?;
        let path = path
            .parse::<PathBuf>()
            .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
        std::fs::remove_file(path)
            .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
        rt.r#return(())
    }

    #[trilogy_derive::proc(crate_name=crate)]
    pub fn remove_dir(rt: Runtime, path: Value) -> Result<()> {
        let path = rt.typecheck::<String>(path)?;
        let path = path
            .parse::<PathBuf>()
            .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
        std::fs::remove_dir(path)
            .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
        rt.r#return(())
    }

    #[trilogy_derive::proc(crate_name=crate)]
    pub fn remove_dir_all(rt: Runtime, path: Value) -> Result<()> {
        let path = rt.typecheck::<String>(path)?;
        let path = path
            .parse::<PathBuf>()
            .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
        std::fs::remove_dir_all(path)
            .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
        rt.r#return(())
    }

    #[trilogy_derive::module(crate_name=crate)]
    impl File {
        #[trilogy_derive::proc(crate_name=crate)]
        fn open(self, rt: Runtime, options: Value) -> Result<()> {
            let options = rt.typecheck::<Array>(options)?;
            let mut open_options = std::fs::File::options();
            for opt in options.into_iter() {
                match opt {
                    Value::Atom(atom) if atom.as_ref() == "read" => {
                        open_options.read(true);
                    }
                    Value::Atom(atom) if atom.as_ref() == "write" => {
                        open_options.write(true);
                    }
                    Value::Atom(atom) if atom.as_ref() == "create" => {
                        open_options.write(true).create(true);
                    }
                    Value::Atom(atom) if atom.as_ref() == "create_new" => {
                        open_options.write(true).create_new(true);
                    }
                    Value::Atom(atom) if atom.as_ref() == "append" => {
                        open_options.append(true);
                    }
                    Value::Atom(atom) if atom.as_ref() == "truncate" => {
                        open_options.write(true).truncate(true);
                    }
                    er => {
                        return Err(rt.runtime_error(
                            rt.r#struct("FileError", format!("invalid open option `{er}`")),
                        ));
                    }
                }
            }
            let file_handle = open_options
                .open(&self.path)
                .map_err(|er| rt.runtime_error(rt.r#struct("FsError", er.to_string())))?;
            *self.file.lock().unwrap() = Some(file_handle);
            rt.r#return(self)
        }

        #[trilogy_derive::proc(crate_name=crate)]
        fn close(self, rt: Runtime) -> crate::Result<()> {
            self.file.lock().unwrap().take();
            rt.r#return(self)
        }
    }
}
