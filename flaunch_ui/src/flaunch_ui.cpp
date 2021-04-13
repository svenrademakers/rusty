#include "nanogui/nanogui.h"
#include <vector>

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

    void init()
    {
        nanogui::init();
    }

    void mainloop()
    {
        Screen *screen = new Screen(Vector2i(500, 700), "Flaunch - Sven Rademakers [devbuild]");
        Button *testbtn = new Button(screen, "JONGEN");

        // bool enabled = true;
        // FormHelper *gui = new FormHelper(screen);
        // ref<Window> window = gui->addWindow(Eigen::Vector2i(10, 10), "Form helper example");
        // gui->addGroup("Basic types");
        // gui->addVariable("bool", bvar);
        // gui->addVariable("string", strval);

        // gui->addGroup("Validating fields");
        // gui->addVariable("int", ivar)->setSpinnable(true);
        // gui->addVariable("float", fvar);
        // gui->addVariable("double", dvar)->setSpinnable(true);

        // gui->addGroup("Complex types");
        // gui->addVariable("Enumeration", enumval, enabled)
        //     ->setItems({"Item 1", "Item 2", "Item 3"});
        // gui->addVariable("Color", colval)
        //     ->setFinalCallback([](const Color &c) {
        //         std::cout << "ColorPicker Final Callback: ["
        //                   << c.r() << ", "
        //                   << c.g() << ", "
        //                   << c.b() << ", "
        //                   << c.w() << "]" << std::endl;
        //     });

        // gui->addGroup("Other widgets");
        // gui->addButton("A button", []() { std::cout << "Button pressed." << std::endl; });

        screen->setVisible(true);
        screen->performLayout();

        nanogui::mainloop();
        nanogui::shutdown();
    }

    struct Script
    {
        std::string name;
    };

    static std::vector<Script>
        ourScripts;

    int add_script(const char *name)
    {
        ourScripts.push_back({name = name});
        return ourScripts.size() - 1;
    }
}