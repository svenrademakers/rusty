#include "nanogui/nanogui.h"
#include <vector>
#include <sstream>
using namespace nanogui;

namespace ui
{
    static Screen *ourScreen = nullptr;

    void init(const char *version, const char *build_date)
    {
        std::stringstream ss;
        ss << "Flaunch - Sven Rademakers [" << version << "][" << build_date << "][devbuild]";
        nanogui::init();
        ourScreen = new Screen(Vector2i(500, 700), ss.str());
    }

    void mainloop()
    {
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

        ourScreen->setVisible(true);
        ourScreen->performLayout();

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
        new nanogui::Button(ourScreen, name);
    }
}