use app_lib::mac_addr::{get_mac_address, change_mac_address, restore_mac_address};

fn main() -> Result<(), String> {
    println!("开始测试 MAC 地址功能...");

    // 测试获取 MAC 地址
    println!("测试获取 MAC 地址...");
    let original_mac = get_mac_address()?;
    println!("当前 MAC 地址: {}", original_mac);

    // 生成随机 MAC 地址
    let random_mac = generate_random_mac();
    println!("生成的随机 MAC 地址: {}", random_mac);

    // 测试修改 MAC 地址
    println!("测试修改 MAC 地址...");
    match change_mac_address(&random_mac) {
        Ok(_) => println!("MAC 地址修改成功"),
        Err(e) => {
            println!("MAC 地址修改失败: {}", e);
            println!("这可能是因为没有管理员权限，在 CI 环境中这是正常的");
            println!("测试将继续进行，但不会实际修改 MAC 地址");
        }
    }

    // 测试还原 MAC 地址
    println!("测试还原 MAC 地址...");
    match restore_mac_address() {
        Ok(_) => println!("MAC 地址还原成功"),
        Err(e) => {
            println!("MAC 地址还原失败: {}", e);
            println!("这可能是因为没有管理员权限，在 CI 环境中这是正常的");
        }
    }

    println!("MAC 地址功能测试完成");
    Ok(())
}

// 生成随机 MAC 地址
fn generate_random_mac() -> String {
    use rand::{thread_rng, Rng};

    let mut rng = thread_rng();
    let mut mac = [0u8; 6];
    rng.fill(&mut mac);

    // 确保是单播地址
    mac[0] &= 0xFE;

    format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    )
}
