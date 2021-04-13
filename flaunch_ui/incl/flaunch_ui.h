#ifndef NANOCUI_H
#define NANOCUI_H

#ifdef __cplusplus
extern "C"
{
#endif

    void init();
    void mainloop();

    int add_script(const char *name);

#ifdef __cplusplus
}
#endif

#endif