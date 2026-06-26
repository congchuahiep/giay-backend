Plan đơn giản ^^

# Workspace

- [x] **Migrations #1**
    - Viết file migration tạo bảng `workspace` và `workspace_membership`.
    - Định nghĩa Enum `MembershipRole` trong CSDL.
- [x] **SeaORM Entities**
    - Chạy `sea-orm-cli generate entity` để sinh các file model cho Workspace.
- [x] **Xây dựng Domain Module**
    - Tạo thư mục `api/src/workspace/` (thuộc cấp kiến trúc phẳng như `auth`, `user`).
    - Cấ`dto.rs`, `handler.rs`, `router.rs`, `service.rs`.
- [x] **Implement Extractors**
    - Tạo file `api/src/workspace/extractor.rs`.
    - Code logic cho `ActiveWorkspace` và các Role Extractors.
- [x] **Phát triển APIs**
    - API tạo Workspace mới.
    - API lấy danh sách Workspace của User đang đăng nhập.
    - API cập nhật/xóa thành viên.
- [x] **Migrations #2**
    - [x] Viết file migration tạo bảng `invitation`
- [x] **API Mời Thành Viên (Invitation)**
    - [x] API xem danh sách lời mời của workspace
    - [x] API mời thành viên (Invitation) hoặc thêm thành viên vào Workspace, gửi mail
    - [x] API chấp nhận lời mời (Accept Invitation)
    - [x] API để người được mời xem thông tin lời mời (public).
    - [x] API để gửi lại lời mời.
    - [x] API huỷ lời mời (Cancel Invitation).
- [ ] **API Quản lý thành viên trong workspace**
    - [ ] API Xem danh sách thành viên
    - [ ] API Đuổi thành viên khỏi workspace.
    - [ ] API Cập nhật thông tin thành viên trong workspace.
    - [ ] API Tự rời khỏi workspace

# Auth

- [ ] Lưu trữ `user_session` bằng cache thay vì lưu trữ trực tiếp trong cơ sở dữ liệu.

# Page

- [ ] **Migrations**
    - Viết tạo bảng `page` với primary key là `id` + `workspace_id`
- [ ] **SeaORM Entities**
    - Chạy `sea-orm-cli generate entity` để sinh các file model cho Page.
- [ ] **Thêm trait extension cho phép workspace bound**
