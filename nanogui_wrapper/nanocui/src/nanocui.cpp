
#include "nanogui/nanogui.h"

extern "C"
{
    void init()
    {
        nanogui::init();
    }

    void run()
    {
        nanogui::mainloop();
        nanogui::shutdown();
    }
}