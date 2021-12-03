#![cfg(target_os = "windows")]

use crate::client::graphics::parameters;
use anyhow::Result;
use defer_lite::defer;
use opengl::{gl::{self, Gl}, wgl::{self, Wgl}};
use std::{
    ffi::CString,
    io::Error,
    mem::{MaybeUninit, size_of},
    os::raw::{c_int, c_void},
    ptr::null,
};
use windows::Win32::{
    Foundation::{LPARAM, LRESULT, HINSTANCE, HWND, PSTR, WPARAM},
    Graphics::{
        Gdi::{
            GetDC,
            HBRUSH,
            HDC,
            PFD_MAIN_PLANE,
            PFD_SUPPORT_OPENGL,
            PFD_TYPE_RGBA,
            ReleaseDC,
        },
        OpenGL::{
            ChoosePixelFormat,
            DescribePixelFormat,
            HGLRC,
            PIXELFORMATDESCRIPTOR,
            SetPixelFormat,
            wglCreateContext,
            wglDeleteContext,
            wglGetProcAddress,
            wglMakeCurrent,
        },
    },
    System::LibraryLoader::{
        FreeLibrary,
        GetModuleHandleA,
        GetProcAddress,
        LoadLibraryA,
    },
    UI::WindowsAndMessaging::{
        CS_OWNDC,
        CW_USEDEFAULT,
        CreateWindowExA,
        DefWindowProcA,
        DestroyWindow,
        HCURSOR,
        HICON,
        HMENU,
        RegisterClassA,
        SW_SHOWDEFAULT,
        ShowWindow,
        UnregisterClassA,
        WINDOW_EX_STYLE,
        WINDOW_STYLE,
        WNDCLASSA,
        WS_CAPTION,
        WS_CLIPCHILDREN,
        WS_CLIPSIBLINGS,
        WS_SYSMENU,
    },
};

macro_rules! assert_windows
{
    ($condition:expr) => {
        if !($condition) {
            return Err(Error::last_os_error().into());
        }
    };
}

/// Create a window with a suitable current OpenGL context.
pub unsafe fn with_environment<F, R>(then: F) -> Result<R>
    where F: FnOnce(&Gl) -> Result<R>
{
    let instance = GetModuleHandleA(PSTR::default());
    assert_windows!(instance.0 != 0);

    let opengl32 = PSTR("OPENGL32.DLL\0".as_ptr() as _);
    let opengl32 = LoadLibraryA(opengl32);
    assert_windows!(opengl32.0 != 0);
    defer! { FreeLibrary(opengl32); }

    let window_class_name = PSTR("blok_main\0".as_ptr() as _);

    let window_class = WNDCLASSA{
        style:         CS_OWNDC,
        lpfnWndProc:   Some(main_wnd_proc),
        cbClsExtra:    0,
        cbWndExtra:    0,
        hInstance:     instance,
        hIcon:         HICON::default(),
        hCursor:       HCURSOR::default(),
        hbrBackground: HBRUSH::default(),
        lpszMenuName:  PSTR::default(),
        lpszClassName: window_class_name,
    };
    let window_class_atom = RegisterClassA(&window_class);
    assert_windows!(window_class_atom != 0);
    defer! { UnregisterClassA(window_class_name, instance); }

    let window = CreateWindowExA(
        /* dwexstyle    */ WINDOW_EX_STYLE::default(),
        /* lpclassname  */ window_class_name,
        /* lpwindowname */ PSTR("Blok\0".as_ptr() as _),
        /* dwstyle      */ WS_CAPTION | WS_CLIPCHILDREN | WS_CLIPSIBLINGS | WS_SYSMENU,
        /* x            */ CW_USEDEFAULT,
        /* y            */ CW_USEDEFAULT,
        /* nwidth       */ 640,
        /* nheight      */ 480,
        /* hwndparent   */ HWND::default(),
        /* hmenu        */ HMENU::default(),
        /* hinstance    */ instance,
        /* lpparam      */ null(),
    );
    assert_windows!(window.0 != 0);
    defer! { DestroyWindow(window); }

    let device_context = GetDC(window);
    assert_windows!(device_context.0 != 0);
    defer! { ReleaseDC(window, device_context); }

    let opengl_context = with_dummy(instance, opengl32, |dummy_dc, dummy_wgl| {

        let pixel_format_attributes: &[c_int] = &[
            wgl::DRAW_TO_WINDOW_ARB as _, gl::TRUE as _,
            wgl::SUPPORT_OPENGL_ARB as _, gl::TRUE as _,
            wgl::DOUBLE_BUFFER_ARB  as _, gl::TRUE as _,
            wgl::PIXEL_TYPE_ARB     as _, wgl::TYPE_RGBA_ARB as _,
            wgl::COLOR_BITS_ARB     as _, parameters::pixel_format::COLOR_BITS,
            wgl::ALPHA_BITS_ARB     as _, parameters::pixel_format::ALPHA_BITS,
            wgl::DEPTH_BITS_ARB     as _, parameters::pixel_format::DEPTH_BITS,
            wgl::STENCIL_BITS_ARB   as _, parameters::pixel_format::STENCIL_BITS,

            // This parameter is why we cannot use ChoosePixelFormat,
            // and have to use wglChoosePixelFormatARB instead.
            wgl::ACCELERATION_ARB   as _, wgl::FULL_ACCELERATION_ARB as _,

            0,
        ];

        let mut pixel_format = MaybeUninit::uninit();
        let mut num_pixel_formats = MaybeUninit::uninit();
        let chose_pixel_format = dummy_wgl.ChoosePixelFormatARB(
            /* hdc           */ dummy_dc.0 as wgl::types::HDC,
            /* piAttribIList */ pixel_format_attributes.as_ptr(),
            /* pfAttribFList */ null(),
            /* nMaxFormats   */ 1,
            /* piFormats     */ pixel_format.as_mut_ptr(),
            /* nNumFormats   */ num_pixel_formats.as_mut_ptr(),
        );
        assert_windows!(chose_pixel_format != 0);
        let num_pixel_formats = num_pixel_formats.assume_init();
        if num_pixel_formats == 0 {
            todo!();
        }
        let pixel_format = pixel_format.assume_init();

        let mut pixel_format_descriptor = MaybeUninit::uninit();
        let described_pixel_format = DescribePixelFormat(
            dummy_dc,
            pixel_format,
            size_of::<PIXELFORMATDESCRIPTOR>() as _,
            pixel_format_descriptor.as_mut_ptr(),
        );
        assert_windows!(described_pixel_format != 0);
        let pixel_format_descriptor = pixel_format_descriptor.assume_init();

        let set_pixel_format = SetPixelFormat(
            device_context,
            pixel_format,
            &pixel_format_descriptor,
        );
        assert_windows!(set_pixel_format.as_bool());

        let context_attributes: &[c_int] = &[
            wgl::CONTEXT_MAJOR_VERSION_ARB as _, parameters::opengl::MAJOR,
            wgl::CONTEXT_MINOR_VERSION_ARB as _, parameters::opengl::MINOR,
            wgl::CONTEXT_PROFILE_MASK_ARB  as _, wgl::CONTEXT_CORE_PROFILE_BIT_ARB as _,
            0,
        ];
        let opengl_context = dummy_wgl.CreateContextAttribsARB(
            /* hDC           */ device_context.0 as _,
            /* hShareContext */ null(),
            /* attribList    */ context_attributes.as_ptr(),
        );
        let opengl_context = HGLRC(opengl_context as _);
        assert_windows!(opengl_context.0 != 0);
        Ok(opengl_context)
    })?;
    defer! { wglDeleteContext(opengl_context); }

    let made_current = wglMakeCurrent(device_context, opengl_context);
    assert_windows!(made_current.as_bool());

    let gl = Gl::load_with(|proc_name| load_callback(instance, proc_name));

    ShowWindow(window, SW_SHOWDEFAULT);

    then(&gl)
}

unsafe extern "system" fn main_wnd_proc(
    param0: HWND,
    param1: u32,
    param2: WPARAM,
    param3: LPARAM,
) -> LRESULT
{
    DefWindowProcA(param0, param1, param2, param3)
}

/// Obtain a low-functionality device context and an OpenGL context.
///
/// These contexts can be used to create a high-functionality OpenGL context.
/// The need for this hoop jumping originates in a limitation of Windows.
/// We need `wglChoosePixelFormatARB` and `wglCreateContextAttribsARB`,
/// but to obtain their addresses we first need an OpenGL context.
/// So we first need to create a dummy OpenGL context.
/// This function also cannot use the main window,
/// because windows can only have their pixel format set once,
/// and we have to set it to some arbitrary pixel format
/// before we can create the dummy OpenGL context for it.
///
/// Both contexts exist for the duration of the callback.
/// They are destroyed once the callback returns.
unsafe fn with_dummy<F, R>(instance: HINSTANCE, opengl32: HINSTANCE, then: F)
    -> Result<R>
    where F: FnOnce(HDC, &Wgl) -> Result<R>
{
    let window_class_name = PSTR("blok_dummy\0".as_ptr() as _);

    let window_class = WNDCLASSA{
        style:         CS_OWNDC,
        lpfnWndProc:   Some(dummy_wnd_proc),
        cbClsExtra:    0,
        cbWndExtra:    0,
        hInstance:     instance,
        hIcon:         HICON::default(),
        hCursor:       HCURSOR::default(),
        hbrBackground: HBRUSH::default(),
        lpszMenuName:  PSTR::default(),
        lpszClassName: window_class_name,
    };
    let window_class_atom = RegisterClassA(&window_class);
    assert_windows!(window_class_atom != 0);
    defer! { UnregisterClassA(window_class_name, instance); }

    // I don’t think the values we supply for these arguments matter much.
    // This is because we won’t use this window for any drawing.
    let window = CreateWindowExA(
        /* dwexstyle    */ WINDOW_EX_STYLE::default(),
        /* lpclassname  */ window_class_name,
        /* lpwindowname */ PSTR::default(),
        /* dwstyle      */ WINDOW_STYLE::default(),
        /* x            */ 0,
        /* y            */ 0,
        /* nwidth       */ 1,
        /* nheight      */ 1,
        /* hwndparent   */ HWND::default(),
        /* hmenu        */ HMENU::default(),
        /* hinstance    */ instance,
        /* lpparam      */ null(),
    );
    assert_windows!(window.0 != 0);
    defer! { DestroyWindow(window); }

    let device_context = GetDC(window);
    assert_windows!(device_context.0 != 0);
    defer! { ReleaseDC(window, device_context); }

    // I don’t think the values we supply for these fields matter much,
    // as long as the pixel format supports OpenGL.
    // This is because we won’t use this pixel format for any drawing.
    let pixel_format_descriptor = PIXELFORMATDESCRIPTOR{
        nSize:           size_of::<PIXELFORMATDESCRIPTOR>() as _,
        nVersion:        1,
        dwFlags:         PFD_SUPPORT_OPENGL,
        iPixelType:      PFD_TYPE_RGBA as _, // XXX: Why?
        cColorBits:      parameters::pixel_format::COLOR_BITS as _,
        cRedBits:        0, // Not used by ChoosePixelFormat.
        cRedShift:       0, // Not used by ChoosePixelFormat.
        cGreenBits:      0, // Not used by ChoosePixelFormat.
        cGreenShift:     0, // Not used by ChoosePixelFormat.
        cBlueBits:       0, // Not used by ChoosePixelFormat.
        cBlueShift:      0, // Not used by ChoosePixelFormat.
        cAlphaBits:      parameters::pixel_format::ALPHA_BITS as _,
        cAlphaShift:     0, // Not used by ChoosePixelFormat.
        cAccumBits:      0,
        cAccumRedBits:   0, // Not used by ChoosePixelFormat.
        cAccumGreenBits: 0, // Not used by ChoosePixelFormat.
        cAccumBlueBits:  0, // Not used by ChoosePixelFormat.
        cAccumAlphaBits: 0, // Not used by ChoosePixelFormat.
        cDepthBits:      parameters::pixel_format::DEPTH_BITS as _,
        cStencilBits:    parameters::pixel_format::STENCIL_BITS as _,
        cAuxBuffers:     0,
        iLayerType:      PFD_MAIN_PLANE as _,
        bReserved:       0, // Not used by ChoosePixelFormat.
        dwLayerMask:     0, // Not used by ChoosePixelFormat.
        dwVisibleMask:   0, // Not used by ChoosePixelFormat.
        dwDamageMask:    0, // Not used by ChoosePixelFormat.
    };
    let pixel_format = ChoosePixelFormat(
        device_context,
        &pixel_format_descriptor,
    );
    assert_windows!(pixel_format != 0);

    // We need to set a pixel format in order to create an OpenGL context,
    // even if we will not use that OpenGL context to do any drawing.
    let set_pixel_format = SetPixelFormat(
        device_context,
        pixel_format,
        &pixel_format_descriptor,
    );
    assert_windows!(set_pixel_format.as_bool());

    let opengl_context = wglCreateContext(device_context);
    assert_windows!(opengl_context.0 != 0);
    defer! { wglDeleteContext(opengl_context); }

    let made_current = wglMakeCurrent(device_context, opengl_context);
    assert_windows!(made_current.as_bool());

    let wgl = Wgl::load_with(|proc_name| load_callback(opengl32, proc_name));

    then(device_context, &wgl)
}

unsafe extern "system" fn dummy_wnd_proc(
    param0: HWND,
    param1: u32,
    param2: WPARAM,
    param3: LPARAM,
) -> LRESULT
{
    DefWindowProcA(param0, param1, param2, param3)
}

/// Callback to pass to the OpenGL loader.
unsafe fn load_callback(opengl32: HINSTANCE, proc_name: &str) -> *const c_void
{
    let proc_name = CString::from_vec_unchecked(proc_name.into());
    let proc_name = PSTR(proc_name.as_ptr() as _);
    match wglGetProcAddress(proc_name) {
        Some(proc_ptr) => proc_ptr as _,
        None =>
            // wglGetProcAddress only loads extension functions.
            // Non-extension functions come from opengl32.dll.
            match GetProcAddress(opengl32, proc_name) {
                Some(proc_ptr) => proc_ptr as _,
                None => null(),
            },
    }
}
