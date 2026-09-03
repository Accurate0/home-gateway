#pragma once
#include "esphome/components/display/display_buffer.h"
#include "esphome/components/font/font.h"

// ── Protected buffer access ───────────────────────────────────────────────────
// DisplayBuffer::buffer_ is protected; this accessor subclass exposes it.
// We never instantiate it — we only reinterpret_cast a live DisplayBuffer* to
// it so we can call raw_buffer(), which accesses buffer_ at the same offset.
struct DisplayBufferGrab : esphome::display::DisplayBuffer {
  const uint8_t* raw_buffer() const { return this->buffer_; }
};

// Retrieve the raw framebuffer from any DisplayBuffer-derived display object.
template<typename T>
inline const uint8_t* grab_raw_buffer(T* display) {
  return reinterpret_cast<DisplayBufferGrab*>(
    static_cast<esphome::display::DisplayBuffer*>(display)
  )->raw_buffer();
}
