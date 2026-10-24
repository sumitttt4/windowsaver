#![allow(non_snake_case)]
use std::{os::windows::ffi::OsStrExt, path::Path};
use windows::{
    core::{implement, Error, Ref, Result, HRESULT, PCWSTR},
    Win32::{
        Foundation::E_ABORT,
        System::Com::{
            CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
            COINIT_APARTMENTTHREADED,
        },
        UI::Shell::*,
    },
};
#[implement(IFileOperationProgressSink)]
struct RecycleGuard { restoring: bool }
impl IFileOperationProgressSink_Impl for RecycleGuard_Impl {
    fn StartOperations(&self) -> Result<()> {
        Ok(())
    }
    fn FinishOperations(&self, result: HRESULT) -> Result<()> {
        result.ok()
    }
    fn PreDeleteItem(&self, flags: u32, _item: Ref<IShellItem>) -> Result<()> {
        if self.restoring || flags & TSF_DELETE_RECYCLE_IF_POSSIBLE.0 as u32 == 0 {
            return Err(Error::new(E_ABORT, "Permanent deletion refused"));
        }
        Ok(())
    }
    fn PostDeleteItem(
        &self,
        _flags: u32,
        _item: Ref<IShellItem>,
        result: HRESULT,
        _created: Ref<IShellItem>,
    ) -> Result<()> {
        result.ok()
    }
    fn PreRenameItem(&self, _: u32, _: Ref<IShellItem>, _: &PCWSTR) -> Result<()> {
        Err(E_ABORT.into())
    }
    fn PostRenameItem(
        &self,
        _: u32,
        _: Ref<IShellItem>,
        _: &PCWSTR,
        _: HRESULT,
        _: Ref<IShellItem>,
    ) -> Result<()> {
        Ok(())
    }
    fn PreMoveItem(
        &self,
        _: u32,
        _: Ref<IShellItem>,
        _: Ref<IShellItem>,
        _: &PCWSTR,
    ) -> Result<()> {
        if self.restoring { Ok(()) } else { Err(E_ABORT.into()) }
    }
    fn PostMoveItem(
        &self,
        _: u32,
        _: Ref<IShellItem>,
        _: Ref<IShellItem>,
        _: &PCWSTR,
        _: HRESULT,
        _: Ref<IShellItem>,
    ) -> Result<()> {
        Ok(())
    }
    fn PreCopyItem(
        &self,
        _: u32,
        _: Ref<IShellItem>,
        _: Ref<IShellItem>,
        _: &PCWSTR,
    ) -> Result<()> {
        Err(E_ABORT.into())
    }
    fn PostCopyItem(
        &self,
        _: u32,
        _: Ref<IShellItem>,
        _: Ref<IShellItem>,
        _: &PCWSTR,
        _: HRESULT,
        _: Ref<IShellItem>,
    ) -> Result<()> {
        Ok(())
    }
    fn PreNewItem(&self, _: u32, _: Ref<IShellItem>, _: &PCWSTR) -> Result<()> {
        Err(E_ABORT.into())
    }
    fn PostNewItem(
        &self,
        _: u32,
        _: Ref<IShellItem>,
        _: &PCWSTR,
        _: &PCWSTR,
        _: u32,
        _: HRESULT,
        _: Ref<IShellItem>,
    ) -> Result<()> {
        Ok(())
    }
    fn UpdateProgress(&self, _: u32, _: u32) -> Result<()> {
        Ok(())
    }
    fn ResetTimer(&self) -> Result<()> {
        Ok(())
    }
    fn PauseTimer(&self) -> Result<()> {
        Ok(())
    }
    fn ResumeTimer(&self) -> Result<()> {
        Ok(())
    }
}
struct ComApartment;
impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() }
    }
}
pub fn recycle_only(path: &Path) -> std::result::Result<(), String> {
    let path = path.to_path_buf();
    std::thread::spawn(move || recycle_on_fresh_thread(&path))
        .join()
        .map_err(|_| "Windows recycle worker failed".to_string())?
}
fn recycle_on_fresh_thread(path: &Path) -> std::result::Result<(), String> {
    let run = || -> Result<()> {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
            let _apartment = ComApartment;
            let operation: IFileOperation =
                CoCreateInstance(&FileOperation, None, CLSCTX_INPROC_SERVER)?;
            // No YES-TO-ALL flag. Explicit recycle mode + early failure + guarded pre-delete callback.
            operation.SetOperationFlags(
                FOFX_RECYCLEONDELETE
                    | FOFX_ADDUNDORECORD
                    | FOFX_EARLYFAILURE
                    | FOF_NOERRORUI
                    | FOF_SILENT
                    | FOF_WANTNUKEWARNING
                    | FOF_NO_CONNECTED_ELEMENTS,
            )?;
            // Shell parsing names do not accept the canonical Win32 verbatim prefix on all Windows builds.
            let raw = path.as_os_str().to_string_lossy();
            let shell = raw.strip_prefix(r"\\?\").unwrap_or(&raw);
            let wide: Vec<u16> = std::ffi::OsStr::new(shell)
                .encode_wide()
                .chain(Some(0))
                .collect();
            let item: IShellItem = SHCreateItemFromParsingName(PCWSTR(wide.as_ptr()), None)?;
            let sink: IFileOperationProgressSink = RecycleGuard { restoring: false }.into();
            operation.DeleteItem(&item, &sink)?;
            operation.PerformOperations()?;
            if operation.GetAnyOperationsAborted()?.as_bool() {
                return Err(Error::new(
                    E_ABORT,
                    "Recycle operation refused or cancelled",
                ));
            }
            Ok(())
        }
    };
    run().map_err(|e| format!("Recycle Bin only: {e}"))
}

// FOF_NO_UI includes automatic confirmation. Never use the library's restore
// defaults without RENAMEONCOLLISION: another process can win the preflight race.
pub fn restore_exact(id: &std::ffi::OsStr, original: &Path) -> std::result::Result<(), String> {
    let id = id.to_owned();
    let original = original.to_owned();
    std::thread::spawn(move || {
        let run = || -> Result<()> {
            unsafe {
                CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
                let _apartment = ComApartment;
                let operation: IFileOperation = CoCreateInstance(&FileOperation, None, CLSCTX_INPROC_SERVER)?;
                operation.SetOperationFlags(FOF_NO_UI | FOF_RENAMEONCOLLISION | FOFX_EARLYFAILURE | FOF_NO_CONNECTED_ELEMENTS)?;
                let source_wide: Vec<u16> = id.encode_wide().chain(Some(0)).collect();
                let source: IShellItem = SHCreateItemFromParsingName(PCWSTR(source_wide.as_ptr()), None)?;
                let parent = original.parent().ok_or_else(|| Error::new(E_ABORT, "Missing restore folder"))?;
                let parent_raw = parent.to_string_lossy();
                let parent_raw = parent_raw.strip_prefix(r"\\?\").unwrap_or(&parent_raw);
                let parent_wide: Vec<u16> = std::ffi::OsStr::new(parent_raw).encode_wide().chain(Some(0)).collect();
                let destination: IShellItem = SHCreateItemFromParsingName(PCWSTR(parent_wide.as_ptr()), None)?;
                let name = original.file_name().ok_or_else(|| Error::new(E_ABORT, "Missing restore name"))?;
                let name_wide: Vec<u16> = name.encode_wide().chain(Some(0)).collect();
                let sink: IFileOperationProgressSink = RecycleGuard { restoring: true }.into();
                operation.MoveItem(&source, &destination, PCWSTR(name_wide.as_ptr()), &sink)?;
                operation.PerformOperations()?;
                if operation.GetAnyOperationsAborted()?.as_bool() {
                    return Err(Error::new(E_ABORT, "Restore refused or cancelled"));
                }
                Ok(())
            }
        };
        run().map_err(|e| format!("Safe restore: {e}"))
    }).join().map_err(|_| "Windows restore worker failed".to_string())?
}

// Called immediately after trash::list initialized COM on the current worker.
// System.FileName is the logical filename, including an Explorer-hidden extension.
pub fn original_name(id: &std::ffi::OsStr) -> std::result::Result<String, String> {
    use windows::Win32::{Storage::EnhancedStorage::PKEY_FileName, System::Com::CoTaskMemFree};
    let run = || -> Result<String> {
        unsafe {
            let wide: Vec<u16> = id.encode_wide().chain(Some(0)).collect();
            let item: IShellItem2 = SHCreateItemFromParsingName(PCWSTR(wide.as_ptr()), None)?;
            let value = item.GetString(&PKEY_FileName)?;
            let name = value.to_string();
            CoTaskMemFree(Some(value.0 as *const std::ffi::c_void));
            let name = name?;
            if name.is_empty() || Path::new(&name).file_name() != Some(std::ffi::OsStr::new(&name)) {
                return Err(Error::new(E_ABORT, "Invalid original filename in Recycle Bin"));
            }
            Ok(name)
        }
    };
    run().map_err(|e| format!("Read exact Recycle Bin filename: {e}"))
}
