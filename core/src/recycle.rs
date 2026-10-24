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
struct RecycleGuard;
impl IFileOperationProgressSink_Impl for RecycleGuard_Impl {
    fn StartOperations(&self) -> Result<()> {
        Ok(())
    }
    fn FinishOperations(&self, result: HRESULT) -> Result<()> {
        result.ok()
    }
    fn PreDeleteItem(&self, flags: u32, _item: Ref<IShellItem>) -> Result<()> {
        if flags & TSF_DELETE_RECYCLE_IF_POSSIBLE.0 as u32 == 0 {
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
        Err(E_ABORT.into())
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
            let sink: IFileOperationProgressSink = RecycleGuard.into();
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
