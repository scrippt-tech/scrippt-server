use crate::models::profile::{education::Education, experience::Experience, skills::Skills};
use crate::models::traits::UpdateFieldId;
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// Profile models
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Profile {
    /// Vector of education objects
    pub education: Vec<Education>,

    /// Vector of experience objects
    pub experience: Vec<Experience>,

    /// Vector of skills objects
    pub skills: Vec<Skills>,

    /// Time of last update
    pub date_updated: Option<i64>,
}

/// Profile value enum used to deserialize a profile field
/// in which the type of the field is unknown.
/// TODO: Switch to external tagging
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
#[serde(rename_all = "snake_case")]
pub enum ProfileValue {
    /// Field ID of the profile object
    FieldId(String),

    /// Name of the experience (e.g. Product Manager)
    Experience(Experience),

    /// Name of the school
    Education(Education),

    /// Name of the skill
    Skills(Skills),
}

impl Profile {
    /// Generate a profile from a JSON string where date updated is the current time and the field ID is for each object is a UUID
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let mut profile: Profile = serde_json::from_str(json)?;

        profile.education.iter_mut().for_each(|education| {
            education.update_field_id(Some(ObjectId::new().to_hex()));
        });
        profile.experience.iter_mut().for_each(|experience| {
            experience.update_field_id(Some(ObjectId::new().to_hex()));
        });
        profile.skills.iter_mut().for_each(|skills| {
            skills.update_field_id(Some(ObjectId::new().to_hex()));
        });

        Ok(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_from_json() {
        let json = r#"{
            "education": [
              {
                "school": "University of Michigan",
                "degree": "Bachelor of Science in Computer Science Engineering",
                "field_of_study": "",
                "current": false,
                "description": "GPA: 3.51 / 4.00"
              }
            ],
            "experience": [
              {
                "name": "Microsoft Corporation",
                "type": "work",
                "at": "C+E Partner Seller Experience Team",
                "current": false,
                "description": "Software Engineer Intern, May 2022 - August 2022\n- Improved Partner Center Marketplace search functionality by fixing deprecated practices on the backend APIs built on ASP.NET.\n- Implemented fixes to the Partner Center Marketplace site content and aesthetics, creating a friendlier user experience for Microsoft Partners.\n- Improved the testing infrastructure by writing additional unit and integration tests that assess the functionality of the new features and fixes applied to the Partner Center’s APIs."
              },
              {
                "name": "University of Michigan",
                "type": "work",
                "at": "College of Engineering",
                "current": true,
                "description": "Instructional Aide for EECS 485 – Web Systems, August 2022 - Present\n- Hosted office hours and lab sessions to provide secondary instruction on web development and distributed computing topics, including server-side dynamic pages, client-side dynamic pages, and MapReduce.\n- Aided in the maintenance of course projects and in the writing of course exams."
              },
              {
                "name": "University of Michigan",
                "type": "work",
                "at": "Center for Academic Innovation",
                "current": false,
                "description": "Software Developer, September 2021 - May 2022\n- Took over the main design and development of an internal web platform that features user authentication, analytics, and content distribution."
              },
              {
                "name": "VOID Tech",
                "type": "work",
                "at": "Executive Board",
                "current": true,
                "description": "VP of Projects, January 2022 - Present\n- Involvement in the management of the organization, which includes member recruiting, project planning, and hosting development, design, or product manager workshops.\n- Responsible for overseeing and managing all projects at a technical and non-technical level, providing support to Project Leads and Role Leads, and leading the integration of tools used in the design and development of the projects.\n\nProduct Manager, September 2021 - August 2022\n- Managed the design and future development of a website for a local non-profit organization that helps organize travel for disabled passengers.\n- Worked closely with the design and development team in the creation of wireframes, mockups, and prototypes of the website, as well as integrating SEO where possible."
              }
            ],
            "skills": [
              {
                "skill": "HTML"
              },
              {
                "skill": "VS Code"
              },
              {
                "skill": "macOS"
              }
            ]
          }"#;

        let profile = Profile::from_json(json).unwrap();

        println!("{:#?}", profile);

        assert_eq!(profile.education.len(), 1);
        assert_eq!(profile.experience.len(), 4);
        assert_eq!(profile.skills.len(), 3);
    }
}
