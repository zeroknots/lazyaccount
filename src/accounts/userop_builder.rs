use std::marker::PhantomData;

pub struct UserOpsBuilder<State> {
    _phantom: PhantomData<State>,
    executions: Vec<Execution>,
}


pub struct SingleExecution;
pub struct BatchExecution;

impl UserOpsBuilder<()> {
    pub(crate) fn new() -> Self {
        Self {
            _phantom: PhantomData,
            executions: Vec::new(),
        }
    }

    pub fn add_execution(
        mut self,
        target: Address,
        value: U256,
        calldata: Bytes,
    ) -> UserOpsBuilder<SingleExecution> {
        self.executions.push(Execution {
            target,
            value,
            callData: calldata,
        });

        UserOpsBuilder {
            _phantom: PhantomData,
            executions: self.executions,
        }
    }
}

impl UserOpsBuilder<SingleExecution> {
    pub(crate) fn add_execution(
        mut self,
        target: Address,
        value: U256,
        calldata: Bytes,
    ) -> UserOpsBuilder<BatchExecution> {
        self.executions.push(Execution {
            target,
            value,
            callData: calldata,
        });

        UserOpsBuilder {
            _phantom: PhantomData,
            executions: self.executions,
        }
    }
}
