# SeaORM Guidelines

Dự án sử dụng SeaORM cho Database Layer. Do đặc thù SeaORM sử dụng công cụ sinh mã nguồn (Code Generator) `sea-orm-cli`, chúng ta phải tuân thủ nghiêm ngặt các quy tắc sau để mã tự sinh không xóa mất mã do con người viết.

## 1. Quy tắc sinh Entity

- **Tuyệt đối KHÔNG** sinh Entity trực tiếp vào thư mục gốc `entity/src/`. Nếu làm vậy, CLI sẽ ghi đè file `lib.rs` và xóa mất toàn bộ cấu hình thủ công của chúng ta.
- **Lệnh chuẩn để sinh Entity:** Phải luôn trỏ output vào thư mục `models/` và thêm cờ `--with-serde both`.
    ```bash
    sea-orm-cli generate entity -o entity/src/models --with-serde both --enum-extra-derives 'utoipa::ToSchema' --enum-extra-attributes 'serde(rename_all = "snake_case")'
    ```

## 2. Quản lý Active Enum (Type-Safe Enums)

Khi Database sử dụng kiểu ENUM (ví dụ: `UserRole` với các giá trị 'admin', 'user'), SeaORM sẽ sinh ra một ActiveEnum tương ứng.

- **KHÔNG ĐƯỢC** viết extension (ví dụ `ext.rs`) để dùng lệnh `.as_str()` chuyển Enum thành String cho các tác vụ như JWT Claims hay API DTO.
- **Giải pháp chuẩn:** Gắn thẳng kiểu `UserRole` vào các struct (như DTO hoặc AccessClaims). Nhờ cờ `--with-serde both` lúc generate, thư viện `serde` sẽ tự động chuyển đổi Enum thành String (chuỗi) khi serialize ra JSON/JWT và ngược lại. Code sẽ Type-safe 100%.
