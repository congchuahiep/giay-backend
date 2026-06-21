# Architecture Guidelines

Dự án `giay-rs` tuân thủ kiến trúc Modular Monolithic và thiết kế đa người thuê (Tenant-Based).

## 1. Cargo Workspaces (Layer Separation)

Dự án được chia làm 3 crate chính nhằm phân tách rõ ràng trách nhiệm:

- **`api/`**: Tầng Application & Business Logic. Chứa cấu hình HTTP Server (Axum), Routing, Middleware, và toàn bộ nghiệp vụ.
- **`entity/`**: Tầng Database Schema. Nơi chứa các Data Models được sinh tự động bởi SeaORM.
- **`migration/`**: Tầng Database Setup. Quản lý lịch sử phiên bản và cập nhật cấu trúc cơ sở dữ liệu.

## 2. Screaming Architecture (Bên trong `api/`)

Thay vì chia thư mục theo mặt cắt kỹ thuật (Controllers, Services, Models), dự án chia theo **Domain (Tính năng)**.
Mọi nghiệp vụ liên quan đến một Domain sẽ được đóng gói trong một thư mục duy nhất.
Ví dụ: `api/src/auth/`, `api/src/user/`, `api/src/workspace/`.

Một Domain tiêu chuẩn sẽ bao gồm:

- `router.rs`: Định tuyến các HTTP endpoint.
- `handler.rs`: Tiếp nhận request từ Client, parse tham số và trả về JSON response.
- `service.rs`: Nơi xử lý cốt lõi của Business logic, tính toán và tương tác với DB.
- `dto.rs`: (Data Transfer Object) Các struct Request/Response của API.

## 3. Kiến trúc Đa người thuê (Tenant-Based)

Dự án được thiết kế hướng tới mô hình tương tự Notion.
Mọi chi tiết thiết kế liên quan đến Partitioning, IDOR Prevention và Workspace Boundaries, vui lòng tham khảo file `tenant-based-plan.md`.
