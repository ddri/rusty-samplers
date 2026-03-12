use crate::types::AkaiProgram;

impl AkaiProgram {
    pub fn to_dspreset_string(&self) -> String {
        // TODO: implement with new types (Task 4)
        String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<DecentSampler minVersion=\"1.0.0\">\n</DecentSampler>\n")
    }
}
