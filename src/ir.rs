#[derive(Debug, Default, Clone)]
pub struct Service {
    pub name: String,
    pub image: Option<String>,
    pub replicas: u32,
    pub ports: Vec<(u16, u16)>,             // host:container
    pub env: Vec<(String, String)>,         // key=value
    pub volumes: Vec<String>,               // "./host:container[:mode]"
}

#[derive(Debug, Default, Clone)]
pub struct Program {
    pub services: Vec<Service>,
}

impl Program {
    pub fn validate(&self) -> Result<(), String> {
        if self.services.is_empty() {
            return Err("no services defined".into());
        }
        for s in &self.services {
            if s.name.is_empty() {
                return Err("service with empty name".into());
            }
            if s.image.is_none() {
                return Err(format!("service '{}' missing required 'image'", s.name));
            }
            if s.replicas == 0 {
                return Err(format!("service '{}' replicas must be >= 1", s.name));
            }
            for (h, c) in &s.ports {
                if *h == 0 || *c == 0 {
                    return Err(format!("service {} has invalid port mapping {} -> {}", s.name, h, c));
                }
            }
        }
        Ok(())
    }
}
