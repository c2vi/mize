// PPC Website UI Module
// This module contains the user interface components for the PPC website

use dioxus::prelude::*;

/// Main PPC website component

pub fn ppc_website() -> Element {
    rsx! {
        div {
            class: "min-h-screen bg-gray-100",

            // Header
            header {
                class: "bg-blue-600 text-white py-4",
                div {
                    class: "container mx-auto px-4",
                    h1 {
                        class: "text-2xl font-bold",
                        "People Corner (PPC)"
                    }
                }
            }

            // Main content
            main {
                class: "container mx-auto px-4 py-8",

                // Hero section
                section {
                    class: "bg-white rounded-lg shadow-md p-6 mb-8",
                    h2 {
                        class: "text-xl font-semibold mb-4 text-blue-600",
                        "Welcome to People Corner"
                    }
                    p {
                        class: "text-gray-700 mb-4",
                        "PPC is a social platform for intentional people who want to get more out of life together."
                    }
                    p {
                        class: "text-gray-700",
                        "We're building a community-driven social media network that brings people together to do cool things, share resources, and support each other."
                    }
                }

                // Mission section
                section {
                    class: "bg-white rounded-lg shadow-md p-6 mb-8",
                    h2 {
                        class: "text-xl font-semibold mb-4 text-blue-600",
                        "Our Mission"
                    }
                    ul {
                        class: "list-disc pl-6 space-y-2",
                        li { "Bring people together to get more out of life" }
                        li { "Create meaningful connections beyond superficial social media" }
                        li { "Support intentional living and personal growth" }
                        li { "Build a platform by people, for people" }
                        li { "Foster real-world meetups and collaborations" }
                        li { "Create a social media for diligent, intentional people" }
                    }
                }

                // Key concepts section
                section {
                    class: "bg-white rounded-lg shadow-md p-6",
                    h2 {
                        class: "text-xl font-semibold mb-4 text-blue-600",
                        "Key Concepts"
                    }
                    ul {
                        class: "list-disc pl-6 space-y-2",
                        li {
                            strong { "Trust Network: " }
                            "A system for building and maintaining trust between members"
                        }
                        li {
                            strong { "Intentional Communities: " }
                            "Both online and physical spaces for like-minded people"
                        }
                        li {
                            strong { "Resource Sharing: " }
                            "Platform for sharing skills, knowledge, and physical resources"
                        }
                        li {
                            strong { "Real Connections: " }
                            "Focus on meaningful relationships over superficial interactions"
                        }
                        li {
                            strong { "Personal Growth: " }
                            "Support for individuals to achieve their life goals"
                        }
                    }
                }
            }

            // Footer
            footer {
                class: "bg-blue-600 text-white py-4 mt-8",
                div {
                    class: "container mx-auto px-4 text-center",
                    p { "© 2024 People Corner. All rights reserved." }
                }
            }
        }
    }
}

/// About page component
pub fn about_page() -> Element {
    rsx! {
        div {
            class: "container mx-auto px-4 py-8",
            h1 {
                class: "text-2xl font-bold mb-4 text-blue-600",
                "About PPC"
            }
            p {
                class: "text-gray-700 mb-4",
                "People Corner (PPC) is more than just a social platform - it's a movement to bring intentional people together to create meaningful connections and achieve more in life."
            }
            p {
                class: "text-gray-700",
                "Our vision is to build a world where people support each other, share resources, and grow together through real-world interactions and digital connections."
            }
        }
    }
}

/// Contact page component
pub fn contact_page() -> Element {
    rsx! {
        div {
            class: "container mx-auto px-4 py-8",
            h1 {
                class: "text-2xl font-bold mb-4 text-blue-600",
                "Contact Us"
            }
            p {
                class: "text-gray-700 mb-4",
                "We'd love to hear from you! Whether you have questions, feedback, or want to get involved, please reach out."
            }
            form {
                class: "max-w-md space-y-4",
                div {
                    label {
                        class: "block text-gray-700 mb-2",
                        "Name"
                    }
                    input {
                        class: "w-full px-3 py-2 border rounded-lg",
                        r#type: "text",
                        placeholder: "Your name"
                    }
                }
                div {
                    label {
                        class: "block text-gray-700 mb-2",
                        "Email"
                    }
                    input {
                        class: "w-full px-3 py-2 border rounded-lg",
                        r#type: "email",
                        placeholder: "Your email"
                    }
                }
                div {
                    label {
                        class: "block text-gray-700 mb-2",
                        "Message"
                    }
                    textarea {
                        class: "w-full px-3 py-2 border rounded-lg h-32",
                        placeholder: "Your message"
                    }
                }
                button {
                    class: "bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700",
                    r#type: "submit",
                    "Send Message"
                }
            }
        }
    }
}
