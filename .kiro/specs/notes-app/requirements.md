# Requirements Document

## Introduction

This document specifies the requirements for a personal notes management web application built using tc-functors (Topology Composer). The application enables authenticated users to create, read, update, and delete personal notes through a responsive, accessible web interface. The backend leverages AWS serverless components including DynamoDB for storage, Cognito for authentication, and AppSync for GraphQL mutations with real-time subscriptions.

## Glossary

- **Notes App**: The web application system for managing personal notes
- **Note**: A user-created text entry with associated metadata (title, text, timestamps, tags, priority)
- **User**: An authenticated individual who owns and manages their personal notes
- **tc-functor**: A namespaced, composable topology of serverless components
- **Topology**: The YAML specification defining serverless entities and their relationships
- **Route**: An HTTP endpoint mapped to a function handler
- **Mutation**: A GraphQL operation that modifies data with optional subscription support
- **Table**: A DynamoDB table for persistent data storage
- **Cognito**: AWS authentication service for user identity management
- **WCAG AA**: Web Content Accessibility Guidelines level AA compliance standard

## Requirements

### Requirement 1

**User Story:** As a user, I want to authenticate with the application, so that I can securely access my personal notes.

#### Acceptance Criteria

1. WHEN a user visits the application without authentication THEN the Notes App SHALL display a sign-in button and redirect to Cognito hosted UI upon click
2. WHEN a user completes Cognito authentication THEN the Notes App SHALL store the user session and display the authenticated interface
3. WHEN an authenticated user clicks sign out THEN the Notes App SHALL clear the session and redirect to the sign-in view
4. IF an authentication token expires THEN the Notes App SHALL prompt the user to re-authenticate
5. WHEN an unauthenticated request reaches a protected route THEN the Notes App SHALL return a 401 Unauthorized response

### Requirement 2

**User Story:** As a user, I want to create new notes, so that I can capture information I need to remember.

#### Acceptance Criteria

1. WHEN a user submits a new note with title and text THEN the Notes App SHALL create a note record with a unique ID, user ID, timestamps, and default priority
2. WHEN a user attempts to create a note with an empty title THEN the Notes App SHALL reject the request and return a validation error
3. WHEN a note is created THEN the Notes App SHALL persist the note to DynamoDB with the user's ID as the partition key
4. WHEN a note is successfully created THEN the Notes App SHALL publish the note to GraphQL subscribers

### Requirement 3

**User Story:** As a user, I want to view all my notes, so that I can browse and find information I previously saved.

#### Acceptance Criteria

1. WHEN a user requests their notes list THEN the Notes App SHALL return all notes belonging to that user sorted by updated timestamp descending
2. WHEN a user has no notes THEN the Notes App SHALL return an empty list
3. WHEN displaying notes THEN the Notes App SHALL show title, text preview, tags, priority, and updated timestamp for each note

### Requirement 4

**User Story:** As a user, I want to view a single note's full details, so that I can read the complete content.

#### Acceptance Criteria

1. WHEN a user requests a specific note by ID THEN the Notes App SHALL return the complete note including all fields
2. IF a user requests a note that does not exist THEN the Notes App SHALL return a 404 Not Found error
3. IF a user requests a note belonging to another user THEN the Notes App SHALL return a 403 Forbidden error

### Requirement 5

**User Story:** As a user, I want to update my notes, so that I can correct or add information.

#### Acceptance Criteria

1. WHEN a user submits an update to an existing note THEN the Notes App SHALL update the specified fields and set a new updated timestamp
2. IF a user attempts to update a note that does not exist THEN the Notes App SHALL return a 404 Not Found error
3. IF a user attempts to update a note belonging to another user THEN the Notes App SHALL return a 403 Forbidden error
4. WHEN a note is successfully updated THEN the Notes App SHALL publish the updated note to GraphQL subscribers

### Requirement 6

**User Story:** As a user, I want to delete notes I no longer need, so that I can keep my notes organized.

#### Acceptance Criteria

1. WHEN a user requests deletion of a note THEN the Notes App SHALL remove the note from DynamoDB
2. IF a user attempts to delete a note that does not exist THEN the Notes App SHALL return a 404 Not Found error
3. IF a user attempts to delete a note belonging to another user THEN the Notes App SHALL return a 403 Forbidden error
4. WHEN a note is successfully deleted THEN the Notes App SHALL publish the deletion event to GraphQL subscribers

### Requirement 7

**User Story:** As a user, I want to organize notes with tags, so that I can categorize and filter my notes.

#### Acceptance Criteria

1. WHEN a user creates or updates a note with tags THEN the Notes App SHALL store the tags as a list of strings
2. WHEN a user requests notes filtered by tag THEN the Notes App SHALL return only notes containing that tag
3. WHEN displaying a note THEN the Notes App SHALL show all associated tags

### Requirement 8

**User Story:** As a user, I want to set priority levels on notes, so that I can identify important items.

#### Acceptance Criteria

1. WHEN a user creates or updates a note with a priority THEN the Notes App SHALL store the priority value (low, medium, high)
2. WHEN a user does not specify a priority THEN the Notes App SHALL default to medium priority
3. WHEN displaying notes THEN the Notes App SHALL visually distinguish priority levels

### Requirement 9

**User Story:** As a user, I want real-time updates when my notes change, so that I can see changes across devices immediately.

#### Acceptance Criteria

1. WHEN a note is created, updated, or deleted THEN the Notes App SHALL broadcast the change via GraphQL subscription
2. WHEN a client subscribes to note changes THEN the Notes App SHALL deliver updates only for notes belonging to that user
3. WHEN a subscription connection is established THEN the Notes App SHALL authenticate the subscriber via Lambda authorizer

### Requirement 10

**User Story:** As a user, I want a responsive interface that works on all devices, so that I can manage notes from my phone or computer.

#### Acceptance Criteria

1. WHEN the viewport width is less than 768 pixels THEN the Notes App SHALL display a single-column mobile layout
2. WHEN the viewport width is 768 pixels or greater THEN the Notes App SHALL display a multi-column desktop layout
3. WHEN the user interacts with the interface THEN the Notes App SHALL provide touch-friendly targets of at least 44x44 pixels on mobile

### Requirement 11

**User Story:** As a user with accessibility needs, I want the application to be accessible, so that I can use it with assistive technologies.

#### Acceptance Criteria

1. WHEN rendering interactive elements THEN the Notes App SHALL provide appropriate ARIA labels and roles
2. WHEN the user navigates with keyboard THEN the Notes App SHALL support Tab, Enter, and Escape key interactions with visible focus indicators
3. WHEN displaying content THEN the Notes App SHALL maintain a minimum color contrast ratio of 4.5:1 for text
4. WHEN form validation fails THEN the Notes App SHALL announce errors to screen readers using aria-live regions
