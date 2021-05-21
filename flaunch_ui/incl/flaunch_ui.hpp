#pragma once
#include <cstdint>

namespace ui
{
    void init(const char *version, const char *build_date);
    void mainloop();

    void script_change_new(uint64_t key, const char* name);
    void script_change_delete(uint64_t key);
}
