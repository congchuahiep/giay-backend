# Project Rules

- MỌI TÁC VỤ LIÊN QUAN ĐẾN KIẾN TRÚC HOẶC THÊM TÍNH NĂNG MỚI ĐỀU PHẢI ĐỌC THƯ MỤC `docs/` TRƯỚC TIÊN ĐỂ HIỂU ĐƯỢC MÔ HÌNH KIẾN TRÚC VÀ CÁC QUYẾT ĐỊNH TRƯỚC ĐÓ.
- Cụ thể, bạn bắt buộc phải đọc file `docs/architecture.md`, `docs/seaorm.md` và `docs/tenant-based-plan.md`.
- File `entity/src/lib.rs` không bị ghi đè bởi `sea-orm-cli` vì output đã được thiết lập là `entity/src/models/`.
- Khi thiết lập cấu hình OpenAPI, sử dụng mô hình "Chia để trị", lưu tại file `swagger.rs` tương ứng của mỗi domain và merge lại ở `lib.rs` hoặc `core/swagger.rs`.
- KHÔNG tạo crate riêng cho mỗi domain, project đang sử dụng Screaming Architecture (Domain Modules) nằm bên trong `api/src/`.
- KHÔNG dùng `ext.rs` để ép kiểu Enum thành String cho JWT/DTO. Thay vào đó, khai báo trực tiếp Type là DB Active Enum (`entity::sea_orm_active_enums::UserRole`), để `serde` tự động serialize.
- ĐỐI VỚI NHỮNG DOMAIN LỚN (ví dụ workspace), áp dụng mô hình Feature Folders (Vertical Split): tạo các thư mục tính năng con bên trong domain (ví dụ `api/src/workspace/invitation/`) chứa cả `handler`, `service`, `dto` của tính năng đó.
- QUY TẮC "PRAGMATIC" CHO HANDLER VÀ SERVICE: KHÔNG tạo `service.rs` hoặc hàm service cho những API CRUD đơn giản (chỉ lấy dữ liệu 1-2 dòng bằng SeaORM hoặc xóa 1 dòng). Viết thẳng DB Query vào trong Handler. CHỈ tạo Service khi logic phức tạp, gọi nhiều bảng, hoặc có logic cần Tái Sử Dụng ở nhiều API/Cronjob khác nhau (như gửi email, resend invitation).
