macro_rules! struct_events {
    (
        keyboard: { $( $k_alias:ident : $k_event:ident ),* },
        else: { $( $e_alias:ident : $e_event:pat ),* }
    ) => {

        pub struct ImmediateEvents {
            $( pub $k_alias : Option<bool> , )*
            $( pub $e_alias : bool ),*
        }

        impl ImmediateEvents {
            pub fn new() -> ImmediateEvents {
                ImmediateEvents {
                    $( $k_alias: None ,)*
                    $( $e_alias: false ),*
                }
            }
        }

        pub struct Events {
            pub now: ImmediateEvents,

            $( pub $k_alias: bool),*
        }

        impl Events {
            pub fn new() -> Events {
                Events {
                    now: ImmediateEvents::new(),

                    $( $k_alias: false),*
                }
            }

            pub fn poll(&mut self, display: &glium::backend::glutin_backend::GlutinFacade) {
                self.now = ImmediateEvents::new();

                for ev in display.poll_events() {
                    use glium::glutin::Event::*;
                    use glium::glutin::VirtualKeyCode::*;
                    use glium::glutin::ElementState::*;

                    match ev {
                        KeyboardInput(state, _, key) => match key {     
                            $(
                                Some($k_event) => {
                                    match state {
                                        Pressed => {
                                            if !self.$k_alias {
                                                self.now.$k_alias = Some(true);
                                            }
                                            self.$k_alias = true;
                                        },
                                        Released => {
                                            self.now.$k_alias = Some(false);
                                            self.$k_alias = false;
                                        }
                                    }
                                }
                            ),*
                            _ => {}
                        },
                        $(
                            $e_event => {
                                self.now.$e_alias = true;
                            }
                        )*,
                        _ => {}
                    }
                }
            }
        }
    }
}