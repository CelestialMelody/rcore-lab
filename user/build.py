import os

base_address = 0x80400000
step = 0x20000
linker = "src/linker.ld"

app_id = 0
apps = os.listdir("build/app")
apps.sort()
chapter = os.getenv("CHAPTER")

# we use `Clink-args=-Ttext=%x` to set the base address of the app
# so do not need to change the linker.ld
for app in apps:
    app = app[: app.find(".")]
    os.system(
        "cargo rustc --bin %s --release -- -Clink-args=-Ttext=%x"
        % (app, base_address + step * app_id)
    )
    print(
        "[build.py] application %s start with address %s"
        % (app, hex(base_address + step * app_id))
    )
    if chapter == "3" or int(chapter) == 3:
        app_id = app_id + 1
