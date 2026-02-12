#include "ailang_runtime.h"

int64_t ailang_rt_identity_i64(int64_t value) {
  return value;
}

bool ailang_rt_identity_bool(bool value) {
  return value;
}

int64_t ailang_rt_time_now(void) {
  return 0;
}

void ailang_rt_log_any() {
}
