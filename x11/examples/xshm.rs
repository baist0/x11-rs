extern crate x11;
extern crate libc;
extern crate rand;

use std::mem;
use std::ptr::{null,null_mut};
use x11::xlib;
use x11::xshm;

pub struct DemoWindow
{
    display: *mut xlib::Display,
    pub win_id: xlib::Window,
    wm_protocols: xlib::Atom,
    wm_delete_window: xlib::Atom
}

impl DemoWindow
{
    pub fn new(display: *mut xlib::Display, root: xlib::Window, 
        width: u32, height: u32) -> DemoWindow
    {
        use std::ffi::CString;
        use std::os::raw::{c_char,c_int,c_uint};
        unsafe
        {
            let mut attributes: xlib::XSetWindowAttributes = mem::zeroed();
    
            let win_id = xlib::XCreateWindow(display, root, 0, 0, width, height,
                0, 24, xlib::InputOutput as c_uint, null_mut(),
                xlib::CWOverrideRedirect | xlib::CWBackPixel | xlib::CWBorderPixel, 
                &mut attributes);
    
            // Set window title.
            let title_str = CString::new("XSHM Example").unwrap();
            xlib::XStoreName(display, win_id, title_str.as_ptr() as *mut c_char);
            // Hook close requests.
            let wm_protocols_str = CString::new("WM_PROTOCOLS").unwrap();
            let wm_delete_window_str = CString::new("WM_DELETE_WINDOW").unwrap();
            let wm_protocols = xlib::XInternAtom(display, 
                wm_protocols_str.as_ptr(), xlib::False);
            let wm_delete_window = xlib::XInternAtom(display, 
                wm_delete_window_str.as_ptr(), xlib::False);
    
            let mut protocols = [wm_delete_window];
    
            xlib::XSetWMProtocols(display, win_id, 
                protocols.as_mut_ptr(), protocols.len() as c_int);
                
            xlib::XSelectInput(display, win_id, 
                xlib::ExposureMask | xlib::KeyPressMask |
                xlib::ButtonPressMask | xlib::StructureNotifyMask);
                
            DemoWindow
            {
                display: display,
                win_id: win_id,
                wm_protocols: wm_protocols,
                wm_delete_window: wm_delete_window
            }
        }
    }
    pub fn show(&mut self)
    {
        unsafe
        {
            // Show window.
            xlib::XMapWindow(self.display, self.win_id);
        }
    }
    
    pub fn prcss_evnt(&mut self) -> bool
    {
        unsafe
        {
            // Event loop
            let mut event: xlib::XEvent = mem::zeroed();

            if xlib::XCheckTypedWindowEvent(self.display, self.win_id,
                xlib::ClientMessage as _, &mut event) != 0 {
                if event.type_ == xlib::ClientMessage
                    && event.client_message.message_type as xlib::Atom == self.wm_protocols
                    && event.client_message.data.get_long(0) as xlib::Atom == self.wm_delete_window
                {
                    return false;
                }
            }

            if xlib::XCheckWindowEvent(self.display, self.win_id,
                xlib::KeyPressMask, &mut event) != 0 {
                if event.type_ == xlib::KeyPress {
                    return false;
                }
            }
        }
        return true;
    }
}

impl Drop for DemoWindow
{
    fn drop(&mut self)
    {
        unsafe { xlib::XDestroyWindow(self.display, self.win_id); }
    }
}

struct Demo
{
    xshm_segment_info: Box<xshm::XShmSegmentInfo>,
    display: *mut xlib::Display,
    demo_window: DemoWindow,
    gc: xlib::GC,
    image: *mut xlib::XImage,
    width: u32,
    height: u32
}

impl Demo
{
    fn create_xshm_sgmnt_inf(size: usize) -> Result<Box<xshm::XShmSegmentInfo>, u8>
    {
        use std::os::raw::{c_char, c_void, c_int};
        use libc::size_t;
        let shmid: c_int = unsafe { libc::shmget(libc::IPC_PRIVATE,
            size as size_t, libc::IPC_CREAT | 0o777) };
        if shmid < 0 {
            return Err(1);
        }
        let shmaddr: *mut libc::c_void = unsafe {
            libc::shmat(shmid, null(), 0) };
        if shmaddr == ((usize::max_value()) as *mut libc::c_void) {
            return Err(2);
        }
        let mut shmidds: libc::shmid_ds = unsafe { mem::zeroed() };
        unsafe { libc::shmctl(shmid, libc::IPC_RMID, &mut shmidds) };
        Ok(Box::new(xshm::XShmSegmentInfo {shmseg: 0, shmid, 
            shmaddr: (shmaddr as *mut c_char), readOnly: 0}))
    }
    
    fn destroy_xshm_sgmnt_inf(seginf: &mut Box<xshm::XShmSegmentInfo>)
    {
        use std::os::raw::c_void;
        unsafe { libc::shmdt(seginf.shmaddr as *mut libc::c_void) };
    }
    
    fn create_xshm_image(dspl: *mut xlib::Display, vsl: *mut xlib::Visual, 
            xshminfo: &mut Box<xshm::XShmSegmentInfo>,
            width: u32, height: u32, depth: u32) -> Result<*mut xlib::XImage, u8>
    {
        unsafe
        {
            let ximg = xshm::XShmCreateImage(dspl, vsl, depth,
                xlib::ZPixmap, null_mut(), 
                xshminfo.as_mut() as *mut _, width, height);
            if ximg == null_mut() {
                return Err(1);
            }
            (*ximg).data = xshminfo.shmaddr;
            Ok(ximg)
        }
    }
    
    fn destroy_xshm_image(ximg: *mut xlib::XImage)
    {
        unsafe
        {
            xlib::XDestroyImage(ximg);
        }
    }

    pub fn new(w: u32, h: u32) -> Demo
    {
        let mut xshminfo = Self::create_xshm_sgmnt_inf(
            (w * h * 4) as usize).unwrap();
        unsafe
        { 
            let dspl = xlib::XOpenDisplay(null());
            if dspl == null_mut() {
                panic!("can't open display");
            }
            let screen_num = xlib::XDefaultScreen(dspl);
            let root_wnd = xlib::XRootWindow(dspl, screen_num);
            let vsl = xlib::XDefaultVisual(dspl, screen_num);
            let mut demo_wnd = DemoWindow::new(dspl, root_wnd, w, h);
            let grph_cntx = xlib::XCreateGC(dspl, demo_wnd.win_id, 0, null_mut());
            let ximg = Self::create_xshm_image(dspl, vsl, 
                &mut xshminfo, w, h, 24).unwrap();
            xshm::XShmAttach(dspl, xshminfo.as_mut() as *mut _);
            xlib::XSync(dspl, xlib::False);
            Demo
            {
                xshm_segment_info: xshminfo,
                display: dspl,
                demo_window: demo_wnd,
                gc: grph_cntx,
                image: ximg,
                width: w,
                height: h
            }
        }
    }

    pub fn start(&mut self)
    {
    }

    pub fn execute(&mut self)
    {
        use std::os::raw::{c_int, c_ulong};
        use rand::Rng;
        unsafe
        {
            self.demo_window.show();
            let mut rng = rand::thread_rng();
            let put_pixel = (*self.image).funcs.put_pixel.unwrap();
            // Main loop
            while self.demo_window.prcss_evnt() {
                let x = rng.gen_range(0, self.width - 1) as c_int;
                let y = rng.gen_range(0, self.height - 1) as c_int;
                let c = rng.gen_range(0, 0x00FFFFFF) as c_ulong;
                put_pixel(self.image, x, y, c);
                xshm::XShmPutImage(self.display, self.demo_window.win_id, 
                    self.gc, self.image, 0, 0, 0, 0, self.width, self.height,
                    xlib::False);
                xlib::XSync(self.display, xlib::False);
            }
        }
    }

    pub fn stop(&mut self)
    {
        unsafe
        { 
            xshm::XShmDetach(self.display, self.xshm_segment_info.as_mut() as *mut _);
            Self::destroy_xshm_image(self.image);
            xlib::XCloseDisplay(self.display);
        }
        Self::destroy_xshm_sgmnt_inf(&mut self.xshm_segment_info);
    }
}

fn main()
{
    let mut demo = Demo::new(800, 600);
    demo.start();
    demo.execute();
    demo.stop();
    println!("Done!");
}
