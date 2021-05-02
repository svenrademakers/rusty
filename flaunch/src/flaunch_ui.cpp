#include "nanogui/nanogui.h"
#include <vector>
#include <sstream>
#include "flaunch_ui.hpp"
#include "image_loader.hpp"
#include "logging.hpp"

using namespace nanogui;

namespace ui
{
    class Menu : public Widget
    {
    public:
        Menu(Widget *parent)
            : Widget(parent)
        {
            BoxLayout *box = new BoxLayout(Orientation::Horizontal, Alignment::Fill);
            this->setLayout(box);

            add_menu_item("Quit", [] {});
        }

        void add_menu_item(const char *text, const std::function<void()> &callback)
        {
            menu_items.push_back(new nanogui::Button(this, text));
            menu_items.back()->setCallback(callback);
        }

    private:
        std::vector<ref<Button>> menu_items;
    };

    static Screen *ourScreen = nullptr;
    static Layout *ourLayout = nullptr;

    void init(const char *version, const char *build_date)
    {
        nanogui::init();

        //glfwWindowHint(GLFW_DECORATED, GLFW_FALSE);

        std::stringstream ss;
        ss << "Flaunch - Sven Rademakers [" << version << "][" << build_date << "][devbuild]";
        ourScreen = new Screen(Vector2i(500, 700), ss.str());
        ourLayout = new BoxLayout(Orientation::Vertical, Alignment::Fill);
        ourScreen->setLayout(ourLayout);

        // Widget *window_header = new Widget(ourScreen);
        // BoxLayout *header_layout = new BoxLayout(Orientation::Horizontal, Alignment::Fill, 0, 5);
        // window_header->setLayout(header_layout);

        // try
        // {
        //     GLuint handle = load_image("C:\\Users\\sven\\Documents\\GitHub\\rusty\\flaunch\\favicon.png");
        //     ImageView *v = new ImageView(window_header, handle);
        // }
        // catch (const char *msg)
        // {
        //     log_error(msg);
        // }
        new ui::Menu(ourScreen);
    }

    void mainloop()
    {
        ourScreen->setVisible(true);
        ourScreen->performLayout();

        nanogui::mainloop();
        nanogui::shutdown();

        delete ourScreen;
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