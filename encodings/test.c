#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>

int main() {
  int f = open("test.txt", O_APPEND | O_WRONLY);
  if (f < 0) {
    printf("error\n");
    return -1;
  }
  
  if (write(f, "CHLOS\n", 6) < 0) {
    printf("cannot write\n");
    return -1;
  }
}
