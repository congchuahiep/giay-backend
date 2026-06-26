# Coding Conventions & Best Practices

Tài liệu này định nghĩa các quy chuẩn viết code (Micro-architecture) và Best Practices được thống nhất trong toàn bộ dự án `giay-rs`. Việc tuân thủ các quy tắc này giúp code base luôn sạch sẽ, dễ bảo trì và hạn chế tối đa các Boilerplate code không cần thiết.

## 1. Mô hình Feature Folders (Vertical Split)

Đối với các Domain lớn có nhiều thực thể con (Sub-entities), ví dụ như `workspace` (bao gồm `workspace` gốc, `invitation`, `membership`), chúng ta không gom chung tất cả vào một file `handler.rs` hay `service.rs` khổng lồ.

Thay vào đó, áp dụng mô hình **Feature Folders (Chia dọc)**:
- Tạo các thư mục con tương ứng với từng tính năng bên trong Domain.
- Mọi thành phần của tính năng đó (`dto.rs`, `handler.rs`, `service.rs`) phải nằm trọn vẹn bên trong thư mục con này.
- Ví dụ: `api/src/workspace/invitation/`

Lợi ích: Tăng tính đóng gói (Encapsulation), dễ dàng tìm kiếm và sửa đổi một tính năng cụ thể mà không sợ ảnh hưởng đến các tính năng khác.

## 2. Handler-Service Pattern: Quy tắc "Thực dụng" (Pragmatic Approach)

Để tránh hội chứng "Anemic Domain" (Tạo ra các tầng Service mỏng dính chỉ làm nhiệm vụ gọi lại Database 1 câu duy nhất), dự án áp dụng quy tắc linh hoạt giữa Handler và Service:

### Khi nào viết thẳng logic vào Handler?
Nếu API của bạn rơi vào trường hợp **Basic CRUD**:
- Chỉ là 1 câu Query Đọc (GET) đơn giản (1-2 dòng code) bằng SeaORM.
- Hoặc Xóa (DELETE) 1 bản ghi duy nhất.
👉 **Quyết định:** Viết trực tiếp DB Query vào trong Handler. KHÔNG CẦN tạo file `service.rs` hay hàm service. Điều này giúp loại bỏ Boilerplate code.

### Khi nào BẮT BUỘC phải tạo Service?
Bạn phải tách logic ra khỏi Handler và đưa vào Service nếu đoạn code đó thỏa mãn 1 trong 3 điều kiện sau:
1. **Tính tái sử dụng (Reusability):** Đoạn nghiệp vụ đó cần được gọi từ nhiều API khác nhau, hoặc từ Background Job (như Cronjob, Task queue). *(Axum Handler không thể gọi trực tiếp một Axum Handler khác).*
2. **Tính phức tạp (Complexity):** Xử lý nghiệp vụ phức tạp, độ dài vượt quá 15-20 dòng, yêu cầu kiểm tra chéo nhiều bảng, hoặc sử dụng Database Transaction.
3. **Tích hợp bên ngoài (Third-party Integration):** Bất kỳ API nào có tương tác với các hệ thống bên ngoài như gửi Email (Mailer), Payment Gateway, v.v.

### Ví dụ thực tế:
- **API `GET /workspace/{slug}/members`**: Trả về danh sách thành viên. Chỉ tốn 1 câu lệnh Inner Join Database -> Nên viết thẳng vào Handler.
- **API `POST /workspace/{slug}/invite`**: Cần check user tồn tại, tạo token, lưu database và gửi Email ngầm -> Bắt buộc nằm trong Service.
