use std::path::Path;

fn main() {
    println!("🧪 Testing rusty-samplers conversion with test file...\n");
    
    let test_file = Path::new("test_sample.akp");
    
    if !test_file.exists() {
        println!("❌ test_sample.akp not found. Run: python3 create_test_akp.py");
        return;
    }
    
    // Test SFZ conversion
    println!("🔄 Testing SFZ conversion...");
    match rusty_samplers::convert_file(test_file, rusty_samplers::OutputFormat::Sfz) {
        Ok(content) => {
            println!("✅ SFZ conversion successful!");
            println!("📄 Output preview:\n{}\n", &content[..content.len().min(200)]);
        }
        Err(error) => {
            println!("❌ SFZ conversion failed: {error}\n");
        }
    }
    
    // Test Decent Sampler conversion  
    println!("🔄 Testing Decent Sampler conversion...");
    match rusty_samplers::convert_file(test_file, rusty_samplers::OutputFormat::DecentSampler) {
        Ok(content) => {
            println!("✅ Decent Sampler conversion successful!");
            println!("📄 Output preview:\n{}\n", &content[..content.len().min(300)]);
        }
        Err(error) => {
            println!("❌ Decent Sampler conversion failed: {error}\n");
        }
    }
    
    println!("🎉 Testing complete! GUI should work with real conversions.");
}