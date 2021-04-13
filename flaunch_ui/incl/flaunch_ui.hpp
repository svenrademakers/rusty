#ifndef NANOCUI_H
#define NANOCUI_H

namespace ui
{
    void init(const char *version, const char *build_date);
    void mainloop();
    int add_script(const char *name);
}

#endif