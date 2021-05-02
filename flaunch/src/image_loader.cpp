#include <sstream>

#include "image_loader.hpp"
#include "lodepng.h"

extern "C"
{
#include <gl/gl.h>
#include <gl/glu.h>
#include <glad/glad.h>
}

GLuint load_image(const char *filename)
{
    GLuint texture;
    glGenTextures(1, &texture);
    glBindTexture(GL_TEXTURE_2D, texture);
    // glTexEnvf(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, (GLfloat)GL_MODULATE);
    glTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_NEAREST);
    glTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    glTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT);
    glTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT);

    std::vector<unsigned char> image;
    unsigned width, height;
    unsigned error = lodepng::decode(image, width, height, filename);
    if (error != 0)
    {
        std::stringstream ss;
        ss << "load_image failure: error " << error << ": " << lodepng_error_text(error);
        throw ss.str();
    }
    gluBuild2DMipmaps(GL_TEXTURE_2D, 3, width, height, GL_RGB, GL_UNSIGNED_BYTE, &*image.begin());

    return texture;
}