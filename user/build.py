import os

base_address = 0x80400000
step = 0x20000 # 128k
linker = "src/linker.ld"

app_id = 0
apps = os.listdir("src/bin") # listdir: get files and filfloders
apps.sort()

for app in apps:
    app = app[: app.find(".")] # get the name of the app
    lines = []
    lines_before = []
    with open(linker, "r") as f:
        for line in f.readlines():
            lines_before.append(line)
            # replace the address of the app
            line = line.replace(hex(base_address), hex(base_address + step * app_id))
            lines.append(line)

    #  write the new linker file
    with open(linker, "w+") as f:
        f.writelines(lines)

    # os.system(
    #     "cargo rustc --bin %s --release -- -Clink-arg=-Ttext=%x" # 使用 --bin 参数来只构建某一个应用
    #     % (app, base_address + step * app_id)
    # )

    os.system(
        "cargo rustc --bin %s --release" # 使用 --bin 参数来只构建某一个应用
        % app
    )

    print(
        "[build.py] application %s strat with address %s"
        % (app, hex(base_address + step * app_id))
    )

    # restore linker.ld
    with open(linker, "w+") as f:
        f.writelines(lines_before)
    app_id = app_id + 1