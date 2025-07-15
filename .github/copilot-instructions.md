# CollapseLoader - AI Coding Assistant Instructions

## Core Commands

### Development
- `npm run dev` - Start Vite development server
- `npm run build` - TypeScript check + Vite build
- `npm run preview` - Preview built application
- `npm run tauri` - Run Tauri commands

### Tauri-specific
- `npm run tauri dev` - Start Tauri development mode
- `npm run tauri build` - Build desktop application

## Architecture Overview

### Technology Stack
- **Frontend**: Vue 3 (Composition API) + TypeScript + Vite
- **Styling**: TailwindCSS v4 + DaisyUI
- **Desktop**: Tauri v2 (Rust backend)
- **Internationalization**: Vue i18n
- **Animations**: GSAP + Lottie
- **HTTP Client**: Axios
- **Icons**: Lucide Vue Next

### Project Structure
```
src/
├── components/          # Vue components organized by domain
│   ├── common/         # Shared utility components
│   ├── core/           # Core app components (modals, setup)
│   ├── features/       # Feature-specific components
│   ├── forms/          # Form components
│   ├── layout/         # Layout components (Sidebar)
│   ├── modals/         # Modal dialogs
│   ├── notifications/  # Toast/notification components
│   └── ui/            # Base UI components
├── composables/        # Vue composables for shared logic
├── services/          # API clients and service layers
├── types/             # TypeScript type definitions
├── views/             # Page-level components
└── i18n/             # Internationalization files

src-tauri/src/
├── commands/          # Tauri command handlers
├── api/              # Backend API logic
├── lib.rs            # Main library entry
└── main.rs           # Application entry point
```

### Key Services
- **apiClient.ts**: Axios wrapper with URL interceptors
- **authClient.ts**: Authentication service
- **modalService.ts**: Global modal management
- **syncService.ts**: Data synchronization
- **userService.ts**: User management
- **notificationService.ts**: System notifications

### External APIs
- CollapseLoader API (`api.collapseloader.org`)
- Authentication service (`auth.collapseloader.org`)
- Configurable via `src/config.ts` (dev/prod URLs)

### Data Flow
- Frontend communicates with Tauri backend via `@tauri-apps/api/core`
- Rust backend handles system operations, file management, client launches
- Vue frontend manages UI state with composables pattern
- Global state managed through services and composables

## Style Guidelines

### TypeScript
- Use strict mode enabled in `tsconfig.json`
- Enable `noUnusedLocals` and `noUnusedParameters`
- Prefer explicit return types for functions
- Use proper typing over `any`

### Vue Components
- Use Composition API with `<script setup>` syntax
- Import types with `import type` when possible
- Organize imports: Vue/framework first, then local imports
- Use `ref` for primitives, `reactive` for objects

### File Organization
- Components in PascalCase (e.g., `GlobalModal.vue`)
- Services in camelCase (e.g., `modalService.ts`)
- Use barrel exports in index files when appropriate
- Keep related files in domain-specific folders

### Naming Conventions
- Composables: `use` prefix (e.g., `useFriends.ts`)
- Services: descriptive names with `Service` suffix
- Components: PascalCase, descriptive names
- Props/events: camelCase

### Error Handling
- Use try-catch blocks for async operations
- Log errors with `console.error`
- Provide user-friendly error messages
- Handle network failures gracefully

### Import Organization
```typescript
// Framework imports first
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Third-party libraries
import axios from 'axios'

// Local imports (services, composables, components)
import { useModal } from '@/services/modalService'
import type { User } from '@/types/user'
```

## Development Notes

### Configuration
- API URLs configurable in `src/config.ts`
- Development mode uses local servers (port 8000/8500)
- TypeScript path alias `@` maps to `./src`

### Key Features
- Desktop client launcher and manager
- Friend system with real-time sync
- User authentication and profiles
- Multilingual support
- System notifications
- Modal-based UI interactions

### Common Patterns
- Use composables for shared reactive logic
- Services for API communication and business logic
- Global modal system via `modalService`
- Toast notifications for user feedback
- Internationalization keys in templates

### Dependencies Management
- Frontend deps in `package.json`
- Rust deps in `src-tauri/Cargo.toml`
- Keep dependencies updated for security

### Build Process
- TypeScript compilation check before build
- Vite handles bundling and optimization
- Tauri packages the desktop application
- Icons and assets processed during build
