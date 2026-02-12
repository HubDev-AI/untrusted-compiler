#ifndef AILANG_RUNTIME_H
#define AILANG_RUNTIME_H

#include <stdbool.h>
#include <stdint.h>

int64_t ailang_rt_identity_i64(int64_t value);
bool ailang_rt_identity_bool(bool value);
int64_t ailang_rt_time_now(void);

#endif
