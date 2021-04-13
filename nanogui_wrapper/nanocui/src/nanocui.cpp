#include "nanogui/nanogui.h"
extern "C"
{
    using namespace nanogui;

    enum test_enum
    {
        Item1 = 0,
        Item2,
        Item3
    };

    bool bvar = true;
    int ivar = 12345678;
    double dvar = 3.1415926;
    float fvar = (float)dvar;
    std::string strval = "A string";
    test_enum enumval = Item2;
    Color colval(0.5f, 0.5f, 0.7f, 1.f);

    int init()
    {
        nanogui::init();

        /* scoped variables */ {
            bool use_gl_4_1 = false; // Set to true to create an OpenGL 4.1 context.
            Screen *screen = nullptr;

            if (use_gl_4_1)
            {
                // NanoGUI presents many options for you to utilize at your discretion.
                // See include/nanogui/screen.h for what all of these represent.
                screen = new Screen(Vector2i(500, 700), "NanoGUI test [GL 4.1]",
                                    /*resizable*/ true, /*fullscreen*/ false, /*colorBits*/ 8,
                                    /*alphaBits*/ 8, /*depthBits*/ 24, /*stencilBits*/ 8,
                                    /*nSamples*/ 0, /*glMajor*/ 4, /*glMinor*/ 1);
            }
            else
            {
                screen = new Screen(Vector2i(500, 700), "NanoGUI test");
            }

            bool enabled = true;
            FormHelper *gui = new FormHelper(screen);
            ref<Window> window = gui->addWindow(Eigen::Vector2i(10, 10), "Form helper example");
            gui->addGroup("Basic types");
            gui->addVariable("bool", bvar);
            gui->addVariable("string", strval);

            gui->addGroup("Validating fields");
            gui->addVariable("int", ivar)->setSpinnable(true);
            gui->addVariable("float", fvar);
            gui->addVariable("double", dvar)->setSpinnable(true);

            gui->addGroup("Complex types");
            gui->addVariable("Enumeration", enumval, enabled)
                ->setItems({"Item 1", "Item 2", "Item 3"});
            gui->addVariable("Color", colval)
                ->setFinalCallback([](const Color &c) {
                    std::cout << "ColorPicker Final Callback: ["
                              << c.r() << ", "
                              << c.g() << ", "
                              << c.b() << ", "
                              << c.w() << "]" << std::endl;
                });

            gui->addGroup("Other widgets");
            gui->addButton("A button", []() { std::cout << "Button pressed." << std::endl; });

            screen->setVisible(true);
            screen->performLayout();
            window->center();

            nanogui::mainloop();
        }

        nanogui::shutdown();
        return 0;
    }
    // static nanogui::Screen *ourMainScreen = nullptr;
    // void init()
    // {
    //     nanogui::init();
    // }

    void mainloop()
    {
        // ourMainScreen = new nanogui::Screen(nanogui::Vector2i(800, 800), "SVEN", true);
        // ourMainScreen->performLayout();
        // ourMainScreen->setVisible(true);

        // nanogui::mainloop();
        // nanogui::shutdown();
    }
}