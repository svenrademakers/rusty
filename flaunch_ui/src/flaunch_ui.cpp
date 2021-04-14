#include "nanogui/nanogui.h"
#include <vector>
#include <sstream>
#include "../incl/flaunch_ui.hpp"
using namespace nanogui;

namespace ui
{
    class Menu : public Widget
    {
    public:
        Menu(Widget *parent)
            : Widget(parent)
        {
            BoxLayout *box = new BoxLayout(Orientation::Horizontal, Alignment::Minimum);
            this->setLayout(box);

            Button *btn1 = new nanogui::Button(this, "this will be a menu bar");
            Button *btn2 = new nanogui::Button(this, "Quit");
        }
    };

    static Screen *ourScreen = nullptr;
    static Layout *ourLayout = nullptr;

    void init(const char *version, const char *build_date)
    {
        nanogui::init();

        std::stringstream ss;
        ss << "Flaunch - Sven Rademakers [" << version << "][" << build_date << "][devbuild]";
        ourScreen = new Screen(Vector2i(500, 700), ss.str());
        ourLayout = new BoxLayout(Orientation::Vertical, Alignment::Fill);
        ourScreen->setLayout(ourLayout);
        new ui::Menu(ourScreen);
    }

    void mainloop()
    {
        ourScreen->setVisible(true);
        ourScreen->performLayout();

        nanogui::mainloop();
        nanogui::shutdown();
    }

    struct Script
    {
        std::string name;
        nanogui::Button *btn;
    };

    static std::vector<std::pair<uint64_t, nanogui::Button *>>
        ourScripts;

    void add_script(uint64_t script_key, const char *name, void (*clicked)(uint64_t))
    {
        ourScripts.push_back(std::make_pair(script_key, new Button(ourScreen, name)));
        ourScripts.back().second->setCallback([script_key, &clicked] { clicked(script_key); });
    }

}