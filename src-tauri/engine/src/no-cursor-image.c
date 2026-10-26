// Loaded into wayvnc with LD_PRELOAD by the hyprland-wsl launcher.
//
// Hyprland draws the pointer into the picture itself. wayvnc also asks it for a separate
// pointer image to hand to the viewer, and for the usual themed pointers Hyprland returns a
// fully transparent one. TigerVNC's Windows viewer turns that into a solid black 24x24
// square on top of the real pointer: Windows draws a cursor that has no visible pixel at all
// as opaque. Sending no pointer image is no better: the viewer then shows the Windows arrow.
//
// So every pointer image wayvnc passes on is replaced with a single black pixel at the
// pointer's tip, which sits on the tip of the pointer Hyprland draws and is the least a
// viewer can be given (an image with any visible pixel is drawn properly by Windows).
// HYPRLAND_WSL_POINTER_IMAGE=1 passes wayvnc's images through unchanged.
//
// neatvnc, the library the call goes to, changed this function between versions:
//   0.x: nvnc_set_cursor(nvnc, nvnc_fb*, width, height, hotspot_x, hotspot_y, damaged)
//   1.x: nvnc_set_cursor(nvnc, nvnc_frame*, hotspot_x, hotspot_y, damaged)
// The version is told by which of nvnc_frame_new (1.x) or nvnc_fb_new (0.x) exists.

#define _GNU_SOURCE
#include <dlfcn.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define FOURCC_ARGB8888 0x34325241u /* "AR24" */

typedef void* (*buffer_new_fn)(uint16_t width, uint16_t height, uint32_t fourcc, uint16_t stride);
typedef void* (*buffer_addr_fn)(const void* buffer);
typedef void (*set_cursor_v1_fn)(void* server, void* frame, uint16_t hx, uint16_t hy, bool damaged);
typedef void (*set_cursor_v0_fn)(void* server, void* fb, uint16_t w, uint16_t h, uint16_t hx, uint16_t hy, bool damaged);

static void* real_set_cursor(void)
{
    static void* real;
    if (!real)
        real = dlsym(RTLD_NEXT, "nvnc_set_cursor");
    return real;
}

static bool pass_through(void)
{
    const char* pass = getenv("HYPRLAND_WSL_POINTER_IMAGE");
    return pass && pass[0] == '1';
}

// Hands neatvnc the one-pixel pointer. Always the same image: neatvnc sends it once and
// skips repeats.
static void send_dot(void* server)
{
    static void* dot;
    static int api = -1; /* 1 = neatvnc 1.x, 0 = 0.x */
    void* real = real_set_cursor();
    if (!real)
        return;

    if (api < 0) {
        buffer_new_fn new_frame = (buffer_new_fn)dlsym(RTLD_DEFAULT, "nvnc_frame_new");
        buffer_new_fn new_fb = (buffer_new_fn)dlsym(RTLD_DEFAULT, "nvnc_fb_new");
        buffer_addr_fn addr = (buffer_addr_fn)dlsym(RTLD_DEFAULT,
                new_frame ? "nvnc_frame_get_addr" : "nvnc_fb_get_addr");
        api = new_frame ? 1 : 0;
        dot = new_frame ? new_frame(1, 1, FOURCC_ARGB8888, 1) : new_fb ? new_fb(1, 1, FOURCC_ARGB8888, 1) : NULL;
        if (dot && addr)
            *(uint32_t*)addr(dot) = 0xff000000u; /* opaque black */
    }
    if (!dot)
        return;

    if (api == 1)
        ((set_cursor_v1_fn)real)(server, dot, 0, 0, true);
    else
        ((set_cursor_v0_fn)real)(server, dot, 1, 1, 0, 0, true);
}

void nvnc_set_cursor(void* server, void* image, long a, long b, long c, long d, long e)
{
    if (pass_through()) {
        void* real = real_set_cursor();
        if (real)
            ((void (*)(void*, void*, long, long, long, long, long))real)(server, image, a, b, c, d, e);
        return;
    }
    send_dot(server);
}

// Older wayvnc versions (0.7, 0.9 with compositors that offer no pointer capture) never pass a
// pointer image on, and the viewer then shows the Windows arrow. They all register their
// display through this call, so the pointer is set from here as well.
void nvnc_add_display(void* server, void* display)
{
    static void (*real)(void*, void*);
    if (!real)
        real = (void (*)(void*, void*))dlsym(RTLD_NEXT, "nvnc_add_display");
    if (real)
        real(server, display);
    if (!pass_through())
        send_dot(server);
}
