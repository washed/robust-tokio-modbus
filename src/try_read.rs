use crate::{
    context::RobustContext,
    types::{Coil, Word},
};
use std::io;
use tokio_modbus::{
    client::Reader, Address, Error as ModbusError, Quantity, Result as ModbusResult,
};

pub(crate) trait TryRead {
    type Result;

    async fn try_read(self, robust_ctx: &RobustContext) -> ModbusResult<Vec<Self::Result>>;
}

pub(crate) struct CoilsRead {
    pub addr: Address,
    pub cnt: Quantity,
}

pub(crate) struct DiscreteInputsRead {
    pub addr: Address,
    pub cnt: Quantity,
}

pub(crate) struct HoldingRegistersRead {
    pub addr: Address,
    pub cnt: Quantity,
}

pub(crate) struct InputRegistersRead {
    pub addr: Address,
    pub cnt: Quantity,
}

pub(crate) struct MultipleRegistersWriteRead<'a> {
    pub read_addr: Address,
    pub read_count: Quantity,
    pub write_addr: Address,
    pub write_data: &'a [Word],
}

impl TryRead for CoilsRead {
    type Result = Coil;
    async fn try_read(self, robust_ctx: &RobustContext) -> ModbusResult<Vec<Self::Result>> {
        let ctx = robust_ctx.ctx.clone();
        let res = {
            let mut ctx_guard = ctx.lock().await;
            let ctx = ctx_guard
                .as_mut()
                .map_err(|e| ModbusError::Transport(io::Error::new(e.kind(), e.to_string())));

            match ctx {
                Ok(ctx) => ctx.read_coils(self.addr, self.cnt).await,
                Err(e) => Err(e),
            }
        };

        if res.is_err() {
            RobustContext::refresh_context(ctx.clone(), &robust_ctx.host, robust_ctx.slave).await;
        }

        res
    }
}

impl TryRead for DiscreteInputsRead {
    type Result = Coil;
    async fn try_read(self, robust_ctx: &RobustContext) -> ModbusResult<Vec<Self::Result>> {
        let ctx = robust_ctx.ctx.clone();
        let res = {
            let mut ctx_guard = ctx.lock().await;
            let ctx = ctx_guard
                .as_mut()
                .map_err(|e| ModbusError::Transport(io::Error::new(e.kind(), e.to_string())));

            match ctx {
                Ok(ctx) => ctx.read_discrete_inputs(self.addr, self.cnt).await,
                Err(e) => Err(e),
            }
        };

        if res.is_err() {
            RobustContext::refresh_context(ctx.clone(), &robust_ctx.host, robust_ctx.slave).await;
        }

        res
    }
}

impl TryRead for HoldingRegistersRead {
    type Result = Word;
    async fn try_read(self, robust_ctx: &RobustContext) -> ModbusResult<Vec<Self::Result>> {
        let ctx = robust_ctx.ctx.clone();
        let res = {
            let mut ctx_guard = ctx.lock().await;
            let ctx = ctx_guard
                .as_mut()
                .map_err(|e| ModbusError::Transport(io::Error::new(e.kind(), e.to_string())));

            match ctx {
                Ok(ctx) => ctx.read_holding_registers(self.addr, self.cnt).await,
                Err(e) => Err(e),
            }
        };

        if res.is_err() {
            RobustContext::refresh_context(ctx.clone(), &robust_ctx.host, robust_ctx.slave).await;
        }

        res
    }
}

impl TryRead for InputRegistersRead {
    type Result = Word;
    async fn try_read(self, robust_ctx: &RobustContext) -> ModbusResult<Vec<Self::Result>> {
        let ctx = robust_ctx.ctx.clone();
        let res = {
            let mut ctx_guard = ctx.lock().await;
            let ctx = ctx_guard
                .as_mut()
                .map_err(|e| ModbusError::Transport(io::Error::new(e.kind(), e.to_string())));

            match ctx {
                Ok(ctx) => ctx.read_input_registers(self.addr, self.cnt).await,
                Err(e) => Err(e),
            }
        };

        if res.is_err() {
            RobustContext::refresh_context(ctx.clone(), &robust_ctx.host, robust_ctx.slave).await;
        }

        res
    }
}

impl<'a> TryRead for MultipleRegistersWriteRead<'a> {
    type Result = Word;
    async fn try_read(self, robust_ctx: &RobustContext) -> ModbusResult<Vec<Self::Result>> {
        let ctx = robust_ctx.ctx.clone();
        let res = {
            let mut ctx_guard = ctx.lock().await;
            let ctx = ctx_guard
                .as_mut()
                .map_err(|e| ModbusError::Transport(io::Error::new(e.kind(), e.to_string())));

            match ctx {
                Ok(ctx) => {
                    ctx.read_write_multiple_registers(
                        self.read_addr,
                        self.read_count,
                        self.write_addr,
                        self.write_data,
                    )
                    .await
                }
                Err(e) => Err(e),
            }
        };

        if res.is_err() {
            RobustContext::refresh_context(ctx.clone(), &robust_ctx.host, robust_ctx.slave).await;
        }

        res
    }
}
