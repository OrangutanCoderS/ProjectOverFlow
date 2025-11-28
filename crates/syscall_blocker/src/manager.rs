use crate::controller::PlatformSysBlocker;
use crate::errors::SysBlockError;
use crate::request::SysBlockRequest;

pub fn handle_request(req: SysBlockRequest) -> Result<(), SysBlockError> {
    req.validate().map_err(SysBlockError::Internal)?;
    PlatformSysBlocker::block_syscall(req.pid, &req.syscall, &req.mode)
}