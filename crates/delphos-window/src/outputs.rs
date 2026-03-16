use std::error::Error;

use delphos_math::IVec2;
use smithay_client_toolkit::{
    delegate_output, delegate_registry,
    output::{OutputHandler, OutputInfo, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
};
use wayland_client::{
    Connection, QueueHandle,
    globals::registry_queue_init,
    protocol::wl_output::{self, WlOutput},
};

pub fn get_main_output(conn: &Connection) -> Result<(IVec2, IVec2, WlOutput), Box<dyn Error>> {
    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let registry_state = RegistryState::new(&globals);
    let output_delegate = OutputState::new(&globals, &qh);

    let mut list_outputs = ListOutputs {
        registry_state,
        output_state: output_delegate,
    };

    event_queue.roundtrip(&mut list_outputs)?;

    let (info, output) = list_outputs
        .get_main_output()
        .expect("Should be at least one output");

    let pos = IVec2::from(info.logical_position.unwrap_or_default());
    let size = IVec2::from(info.logical_size.expect("Unable to determine output size"));

    log::debug!("Main output info:");
    log::debug!("\tname         {}", info.name.unwrap_or(info.model));
    log::debug!("\tlogical pos  {pos:?}");
    log::debug!("\tlogical size {size:?}");

    Ok((pos, size, output))
}

struct ListOutputs {
    registry_state: RegistryState,
    output_state: OutputState,
}

impl ListOutputs {
    fn get_main_output(self) -> Option<(OutputInfo, WlOutput)> {
        self.output_state
            .outputs()
            .filter_map(|out| self.output_state.info(&out).map(|i| (i, out)))
            .next()
    }
}

impl OutputHandler for ListOutputs {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

delegate_output!(ListOutputs);
delegate_registry!(ListOutputs);

impl ProvidesRegistryState for ListOutputs {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers![OutputState];
}
