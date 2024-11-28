use std::{fs::File, os::unix::io::AsFd};

use wayland_client::{
    delegate_noop, event_created_child,
    protocol::{
        wl_buffer, wl_compositor, wl_keyboard, wl_pointer, wl_registry, wl_seat, wl_shm,
        wl_shm_pool, wl_surface,
    },
    Connection, Dispatch, QueueHandle, WEnum,
};

use wayland_protocols::{
    wp::{
        pointer_gestures::zv1::client::{
            zwp_pointer_gesture_pinch_v1, zwp_pointer_gesture_swipe_v1, zwp_pointer_gestures_v1,
        },
        tablet::zv2::client::{
            zwp_tablet_manager_v2, zwp_tablet_pad_v2::ZwpTabletPadV2,
            zwp_tablet_seat_v2::ZwpTabletSeatV2, zwp_tablet_tool_v2::ZwpTabletToolV2,
            zwp_tablet_v2::ZwpTabletV2,
        },
    },
    xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base},
};

pub struct WaylandConnection {
    pub running: bool,
    base_surface: Option<wl_surface::WlSurface>,
    buffer: Option<wl_buffer::WlBuffer>,
    wm_base: Option<xdg_wm_base::XdgWmBase>,
    tablet_manager: Option<zwp_tablet_manager_v2::ZwpTabletManagerV2>,
    pointer_gestures: Option<zwp_pointer_gestures_v1::ZwpPointerGesturesV1>,
    xdg_surface: Option<(xdg_surface::XdgSurface, xdg_toplevel::XdgToplevel)>,
    configured: bool,
}

impl Dispatch<wl_registry::WlRegistry, ()> for WaylandConnection {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name, interface, ..
        } = event
        {
            match dbg!(&interface[..]) {
                "wl_compositor" => {
                    let compositor =
                        registry.bind::<wl_compositor::WlCompositor, _, _>(name, 1, qh, ());
                    let surface = compositor.create_surface(qh, ());
                    state.base_surface = Some(surface);

                    if state.wm_base.is_some() && state.xdg_surface.is_none() {
                        state.init_xdg_surface(qh);
                    }
                }
                "wl_shm" => {
                    let shm = registry.bind::<wl_shm::WlShm, _, _>(name, 1, qh, ());

                    let (init_w, init_h) = (320, 240);

                    let mut file = tempfile::tempfile().unwrap();
                    draw(&mut file, (init_w, init_h));
                    let pool = shm.create_pool(file.as_fd(), (init_w * init_h * 4) as i32, qh, ());
                    let buffer = pool.create_buffer(
                        0,
                        init_w as i32,
                        init_h as i32,
                        (init_w * 4) as i32,
                        wl_shm::Format::Argb8888,
                        qh,
                        (),
                    );
                    state.buffer = Some(buffer.clone());

                    if state.configured {
                        let surface = state.base_surface.as_ref().unwrap();
                        surface.attach(Some(&buffer), 0, 0);
                        surface.commit();
                    }
                }
                "wl_seat" => {
                    registry.bind::<wl_seat::WlSeat, _, _>(name, 1, qh, ());
                }
                "xdg_wm_base" => {
                    let wm_base = registry.bind::<xdg_wm_base::XdgWmBase, _, _>(name, 1, qh, ());
                    state.wm_base = Some(wm_base);

                    if state.base_surface.is_some() && state.xdg_surface.is_none() {
                        state.init_xdg_surface(qh);
                    }
                }
                "zwp_tablet_manager_v2" => {
                    let wp_tablet_manager = registry
                        .bind::<zwp_tablet_manager_v2::ZwpTabletManagerV2, _, _>(name, 1, qh, ());
                    state.tablet_manager = Some(wp_tablet_manager);
                }
                "zwp_pointer_gestures_v1" => {
                    let wp_gestures = registry
                        .bind::<zwp_pointer_gestures_v1::ZwpPointerGesturesV1, _, _>(
                            name,
                            1,
                            qh,
                            (),
                        );
                    state.pointer_gestures = Some(wp_gestures);
                }
                _other => {
                    //dbg!(other);
                }
            }
        }
    }
}

// Ignore events from these object types in this example.
delegate_noop!(WaylandConnection: ignore wl_compositor::WlCompositor);
delegate_noop!(WaylandConnection: ignore wl_surface::WlSurface);
delegate_noop!(WaylandConnection: ignore wl_shm::WlShm);
delegate_noop!(WaylandConnection: ignore wl_shm_pool::WlShmPool);
delegate_noop!(WaylandConnection: ignore wl_buffer::WlBuffer);

fn draw(tmp: &mut File, (buf_x, buf_y): (u32, u32)) {
    use std::{cmp::min, io::Write};
    let mut buf = std::io::BufWriter::new(tmp);
    for y in 0..buf_y {
        for x in 0..buf_x {
            let a = 0xFF;
            let r = min(((buf_x - x) * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
            let g = min((x * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
            let b = min(((buf_x - x) * 0xFF) / buf_x, (y * 0xFF) / buf_y);
            buf.write_all(&[b as u8, g as u8, r as u8, a as u8])
                .unwrap();
        }
    }
    buf.flush().unwrap();
}

impl WaylandConnection {
    pub fn new() -> Self {
        WaylandConnection {
            running: true,
            base_surface: None,
            buffer: None,
            wm_base: None,
            tablet_manager: None,
            pointer_gestures: None,
            xdg_surface: None,
            configured: false,
        }
    }

    fn init_xdg_surface(&mut self, qh: &QueueHandle<WaylandConnection>) {
        let wm_base = self.wm_base.as_ref().unwrap();
        let base_surface = self.base_surface.as_ref().unwrap();

        let xdg_surface = wm_base.get_xdg_surface(base_surface, qh, ());
        let toplevel = xdg_surface.get_toplevel(qh, ());
        toplevel.set_title("A fantastic window!".into());

        base_surface.commit();

        self.xdg_surface = Some((xdg_surface, toplevel));
    }
}

impl Default for WaylandConnection {
    fn default() -> Self {
        Self::new()
    }
}

/// TabletManager doesn't  receive any events, but we still have to implement it
impl Dispatch<zwp_tablet_manager_v2::ZwpTabletManagerV2, ()> for WaylandConnection {
    fn event(
        _state: &mut Self,
        _proxy: &zwp_tablet_manager_v2::ZwpTabletManagerV2,
        _event: <zwp_tablet_manager_v2::ZwpTabletManagerV2 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        todo!()
    }
}

/// When we have a seat we can get the tablet info
impl Dispatch<wl_seat::WlSeat, ()> for WaylandConnection {
    fn event(
        state: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        dbg!(&event);
        //// HERE
        state
            .tablet_manager
            .clone()
            .unwrap()
            .get_tablet_seat(seat, qh, ());
        println!("got tablet seat");

        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(capabilities),
        } = event
        {
            if capabilities.contains(wl_seat::Capability::Keyboard) {
                seat.get_keyboard(qh, ());
            }
            if capabilities.contains(wl_seat::Capability::Pointer) {
                let pointer = seat.get_pointer(qh, ());
                let g = state.pointer_gestures.clone().unwrap();
                g.get_pinch_gesture(&pointer, qh, ());
                g.get_swipe_gesture(&pointer, qh, ());
                //g.get_pinch_gesture(&pointer, qh, ());
            }
        }
    }
}
/// The seat gives access to the keyboard
impl Dispatch<wl_keyboard::WlKeyboard, ()> for WaylandConnection {
    fn event(
        state: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_keyboard::Event::Key { key, .. } = event {
            if key == 1 {
                // ESC key
                state.running = false;
            }
        }
    }
}

/// The seat gives access to the keyboard
impl Dispatch<wl_pointer::WlPointer, ()> for WaylandConnection {
    fn event(
        _state: &mut Self,
        _pointer: &wl_pointer::WlPointer,
        event: wl_pointer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        println!("pointer event!");
        println!("{:?}", event);
    }
}

/// And the tablet, with some indirection
impl Dispatch<ZwpTabletSeatV2, ()> for WaylandConnection {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpTabletSeatV2,
        event: <wayland_protocols::wp::tablet::zv2::client::zwp_tablet_seat_v2::ZwpTabletSeatV2 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        println!("tablet event!");
        println!("{:?}", event);
    }
    ////  ZwpTabletSeat creates more objects that will handle all tablet events
    event_created_child!(WaylandConnection, ZwpTabletSeatV2, [
             0 => (ZwpTabletV2, ()), // Tablet level info, connect disconnect, etc.
             1 => (ZwpTabletToolV2, ()),// tool level info, Pen up, down, move, etc
             2 => (ZwpTabletPadV2, ()), // pad level info, buttons on the tablet?
    ]);
}

impl Dispatch<ZwpTabletV2, ()> for WaylandConnection {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpTabletV2,
        event: <ZwpTabletV2 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _handle: &QueueHandle<Self>,
    ) {
        println!("tablet device event!");
        println!("{:?}", event);
    }
}

impl Dispatch<ZwpTabletToolV2, ()> for WaylandConnection {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpTabletToolV2,
        event: <ZwpTabletToolV2 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        println!("tool event!");
        println!("{:?}", event);
    }
}

impl Dispatch<ZwpTabletPadV2, ()> for WaylandConnection {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpTabletPadV2,
        event: <ZwpTabletPadV2 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        println!("pad event!");
        println!("{:?}", event);
    }
}

impl Dispatch<zwp_pointer_gestures_v1::ZwpPointerGesturesV1, ()> for WaylandConnection {
    fn event(
        _state: &mut Self,
        _proxy: &zwp_pointer_gestures_v1::ZwpPointerGesturesV1,
        event: <zwp_pointer_gestures_v1::ZwpPointerGesturesV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        println!("gesture event!");
        println!("{:?}", event);
    }
}
impl Dispatch<zwp_pointer_gesture_pinch_v1::ZwpPointerGesturePinchV1, ()> for WaylandConnection {
    fn event(
        _state: &mut Self,
        _proxy: &zwp_pointer_gesture_pinch_v1::ZwpPointerGesturePinchV1,
        event: <zwp_pointer_gesture_pinch_v1::ZwpPointerGesturePinchV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        println!("gesture pinch event!");
        println!("{:?}", event);
    }
}
impl Dispatch<zwp_pointer_gesture_swipe_v1::ZwpPointerGestureSwipeV1, ()> for WaylandConnection {
    fn event(
        _state: &mut Self,
        _proxy: &zwp_pointer_gesture_swipe_v1::ZwpPointerGestureSwipeV1,
        event: <zwp_pointer_gesture_swipe_v1::ZwpPointerGestureSwipeV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        println!("gesture swipe event!");
        println!("{:?}", event);
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for WaylandConnection {
    fn event(
        _: &mut Self,
        wm_base: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            wm_base.pong(serial);
        }
    }
}

impl Dispatch<xdg_surface::XdgSurface, ()> for WaylandConnection {
    fn event(
        state: &mut Self,
        xdg_surface: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial, .. } = event {
            xdg_surface.ack_configure(serial);
            state.configured = true;
            let surface = state.base_surface.as_ref().unwrap();
            if let Some(ref buffer) = state.buffer {
                surface.attach(Some(buffer), 0, 0);
                surface.commit();
            }
        }
    }
}

impl Dispatch<xdg_toplevel::XdgToplevel, ()> for WaylandConnection {
    fn event(
        state: &mut Self,
        _: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_toplevel::Event::Close {} = event {
            state.running = false;
        }
    }
}
