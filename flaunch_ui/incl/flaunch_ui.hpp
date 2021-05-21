#pragma once
#include <cstdint>

namespace ui
{
    // enum ScriptEventType
    // {
    //     New,
    //     Deleted,
    //     Modified
    // };

    // void wat(ScriptEventType wat);
    void init(const char *version, const char *build_date);
    void mainloop();

    //void add_script(uint64_t script_key, const char *name, void (*clicked)(uint64_t));
}
