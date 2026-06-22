# Kế Hoạch Triển Khai Kiến Trúc Tenant-Based (Workspace)

Tài liệu này mô tả chi tiết kế hoạch và các quyết định kiến trúc để xây dựng hệ thống Multi-tenant (Workspace-based) tương tự như Notion, Slack, hỗ trợ khả năng phân mảnh dữ liệu (Sharding) và bảo mật tuyệt đối (chống IDOR).

## 1. Thiết Kế Cơ Sở Dữ Liệu (Database Models)

Chúng ta sử dụng mô hình **Shared Database, Shared Schema**, nơi tất cả dữ liệu của mọi Workspace nằm chung trong cùng một CSDL, nhưng được phân tách logic bằng cột `workspace_id`.

### 1.1. Các Bảng Cốt Lõi (Core Tables)
1. **Bảng `workspace`**:
   - `id` (UUIDv7, Primary Key)
   - `name` (String)
   - `slug` (String, giới hạn 255 Unique, Index - Dùng để làm URL, ví dụ: `/w/my-company`)
   - `icon` (String) giới hạn 1 ký tự
   - `owner_id` (UUIDv7, Foreign Key trỏ tới `user.id`)
   - `created_at` (Timestamp)
   - `updated_at` (Timestamp)

2. **Bảng `workspace_membership`**:
   - `workspace_id` (UUIDv7, Foreign Key)
   - `user_id` (UUIDv7, Foreign Key)
   - `role` (Enum: `OWNER`, `MODERATOR`, `MEMBER`, `VIEWER`)
   - *Primary Key kép:* `(workspace_id, user_id)`

### 1.2. Kỹ Thuật Khóa Chính Kép (Composite Primary Keys) và UUIDv7
Để giải quyết bài toán chống trùng lặp ID trong môi trường **Offline-first** (ID do Client sinh ra) và chuẩn bị hạ tầng cho **Database Partitioning / Sharding** khi dữ liệu phình to lên hàng trăm triệu dòng, chúng ta áp dụng các nguyên tắc sau:

1. **Sử dụng UUIDv7 cho toàn bộ hệ thống:**
   - Tất cả các ID (bao gồm `workspace_id`, `page_id`, `block_id`...) BẮT BUỘC phải sử dụng định dạng **UUIDv7**.
   - *Lý do:* UUIDv7 chứa timestamp giúp tuần tự hóa khi Insert, triệt tiêu hoàn toàn vấn đề phân mảnh B-Tree của UUIDv4. Điều này giúp tối ưu cực tốt tốc độ đồng bộ (Bulk Insert) khi Client online trở lại và đẩy một khối lượng lớn dữ liệu Offline lên Server. Đồng thời hỗ trợ sắp xếp theo thời gian mặc định.

2. **Composite Primary Key (Khóa chính kép):**
   - Mọi bảng dữ liệu sinh ra bên trong Workspace (ví dụ: `page`, `block`, `comment`) đều **BẮT BUỘC** phải có `workspace_id`.
   - Sử dụng khóa chính kép: `(workspace_id, entity_id)`.
   - *Ví dụ bảng `page`:* Primary Key là `(workspace_id, id)`. Cấu trúc này khoanh vùng rủi ro va chạm ID xuống mức cách ly hoàn toàn theo Workspace, đồng thời bắt buộc mọi câu Query phải đính kèm `workspace_id`, ngăn chặn lỗi IDOR triệt để.

## 2. Kiến Trúc Phân Quyền Bằng Axum Extractors

Thay vì sử dụng Middleware cồng kềnh, chúng ta tận dụng sức mạnh Type-System của Rust để chặn luồng truy cập trái phép ngay tại cánh cửa Extractor.

### 2.1. Trái tim hệ thống: `ActiveWorkspace` Extractor
Đóng vai trò xác định ngữ cảnh của Request hiện tại:
1. Lấy `workspace_slug` từ URL Path (vd: `/api/w/:workspace_slug/...`).
2. Lấy `AuthenticatedUser` từ Request.
3. Truy vấn Database (hoặc Cache) xem User có tồn tại trong `workspace_membership` của Workspace này không.
4. Trả về cấu trúc `ActiveWorkspace { workspace_id, role }` hoặc ném lỗi `403 Forbidden` nếu không có quyền.

### 2.2. Phân quyền theo Role (Role-based Extractors)
Tạo các Extractor bọc lấy `ActiveWorkspace` để ép kiểu quyền hạn cho từng API:
- `WorkspaceMember`: Quyền truy cập cơ bản.
- `WorkspaceModerator`: Có quyền quản trị nội dung.
- `WorkspaceOwner`: Quyền tối thượng (xóa Workspace, đổi role thành viên).

*Ví dụ áp dụng trong Handler:*
```rust
pub async fn delete_page(
    owner: WorkspaceOwner, // Tự động văng 403 nếu không phải Owner/Moderator
    Path((slug, page_id)): Path<(String, Uuid)>,
) { ... }
```

## 3. Lộ Trình Thực Thi (Execution Roadmap)

Cần thực hiện theo các bước tuần tự sau:

- [ ] **Bước 1: Migrations** 
  - Viết file migration tạo bảng `workspace` và `workspace_membership`.
  - Định nghĩa Enum `MembershipRole` trong CSDL.
- [ ] **Bước 2: SeaORM Entities**
  - Chạy `sea-orm-cli generate entity` để sinh các file model cho Workspace.
- [ ] **Bước 3: Xây dựng Domain Module**
  - Tạo thư mục `api/src/workspace/` (thuộc cấp kiến trúc phẳng như `auth`, `user`).
  - Cấu trúc: `dto.rs`, `handler.rs`, `router.rs`, `service.rs`.
- [ ] **Bước 4: Implement Extractors**
  - Tạo file `api/src/workspace/extractor.rs`.
  - Code logic cho `ActiveWorkspace` và các Role Extractors.
- [ ] **Bước 5: Phát triển APIs**
  - API tạo Workspace mới.
  - API lấy danh sách Workspace của User đang đăng nhập.
  - API mời thành viên (Invitation) hoặc thêm thành viên vào Workspace.
  - API cập nhật/xóa thành viên.

## 4. Tương lai (Future Scalability)
- **Caching**: Lọc và lưu trữ `workspace_context` (chứa role của user) vào Redis với thời gian sống ngắn để giảm tải truy vấn JOIN bảng Membership ở mỗi Request.
- **Partitioning**: Khi bảng `page` vượt mốc 50 triệu dòng, tiến hành cấu hình PostgreSQL Native Partitioning băm bảng theo trường `workspace_id`.
